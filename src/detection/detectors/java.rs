use std::{fs, io, path::Path};

use crate::detection::{DetectionConfidence, LanguageCandidate};
use crate::schema::Language;

/// Detects Java projects from Maven and Gradle build files.
///
/// # Errors
///
/// Returns any I/O error produced while reading Maven or Gradle files.
pub fn detect(target_dir: &Path) -> io::Result<Option<LanguageCandidate>> {
    let pom = target_dir.join("pom.xml");
    let gradle = target_dir.join("build.gradle");
    let gradle_kts = target_dir.join("build.gradle.kts");

    let has_maven = pom.exists();
    let has_gradle = gradle.exists() || gradle_kts.exists();

    if !has_maven && !has_gradle {
        return Ok(None);
    }

    let mut reasons = Vec::new();
    if has_maven {
        reasons.push("found pom.xml".to_string());
    }
    if gradle.exists() {
        reasons.push("found build.gradle".to_string());
    }
    if gradle_kts.exists() {
        reasons.push("found build.gradle.kts".to_string());
    }

    // Detect gradle wrapper
    let has_gradlew = target_dir.join("gradlew").exists()
        || target_dir.join("gradlew.bat").exists();
    if has_gradlew {
        reasons.push("found gradlew (wrapper, system gradle not needed)".to_string());
    }

    // Build tool: mutually exclusive, prefer gradle over maven.
    // If gradle wrapper exists, don't enable system gradle.
    let (gradle_enable, maven_enable) = if has_gradle {
        if has_gradlew {
            (None, None) // wrapper handles gradle, no system gradle needed
        } else {
            (Some(true), None) // system gradle, no maven
        }
    } else if has_maven {
        (None, Some(true))
    } else {
        (None, None)
    };

    let jdk_package = detect_jdk_package(target_dir)?;

    Ok(Some(LanguageCandidate {
        language: Language::Java {
            jdk_package,
            gradle_enable,
            maven_enable,
        },
        confidence: DetectionConfidence::High,
        reasons,
    }))
}

fn detect_jdk_package(target_dir: &Path) -> io::Result<Option<String>> {
    let pom = target_dir.join("pom.xml");
    if pom.exists() {
        let content = fs::read_to_string(&pom)?;
        if let Some(major) = parse_maven_java_version(&content)
            && let Some(pkg) = jdk_package_for_major(&major)
        {
            return Ok(Some(pkg));
        }
    }

    for file in ["build.gradle", "build.gradle.kts"] {
        let path = target_dir.join(file);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            if let Some(major) = parse_gradle_java_version(&content)
                && let Some(pkg) = jdk_package_for_major(&major)
            {
                return Ok(Some(pkg));
            }
        }
    }

    Ok(None)
}

fn parse_maven_java_version(content: &str) -> Option<String> {
    let sanitized = strip_xml_comments(content);
    for tag in [
        "maven.compiler.release",
        "maven.compiler.source",
        "java.version",
    ] {
        if let Some(value) = first_direct_tag_value(&sanitized, tag) {
            return Some(value);
        }
    }
    None
}

fn first_direct_tag_value(content: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let mut search_start = 0;
    while let Some(rel_start) = content[search_start..].find(&start_tag) {
        let abs_start = search_start + rel_start;
        let rest = &content[abs_start + start_tag.len()..];
        if let Some(rel_end) = rest.find(&end_tag) {
            let value = &rest[..rel_end];
            if let Some(direct) = direct_numeric_value(value) {
                return Some(direct);
            }
            search_start = abs_start + start_tag.len() + rel_end + end_tag.len();
        } else {
            break;
        }
    }
    None
}

fn direct_numeric_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains("${") {
        return None;
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn strip_xml_comments(content: &str) -> String {
    let mut result = String::new();
    let mut idx = 0;
    while let Some(start) = content[idx..].find("<!--") {
        let abs_start = idx + start;
        result.push_str(&content[idx..abs_start]);
        if let Some(end_rel) = content[abs_start + 4..].find("-->") {
            idx = abs_start + 4 + end_rel + 3;
        } else {
            idx = content.len();
            break;
        }
    }
    if idx < content.len() {
        result.push_str(&content[idx..]);
    }
    result
}

fn jdk_package_for_major(major: &str) -> Option<String> {
    if major.is_empty() {
        return None;
    }

    if major.chars().all(|c| c.is_ascii_digit()) {
        Some(format!("pkgs.jdk{major}"))
    } else {
        None
    }
}

fn parse_gradle_java_version(content: &str) -> Option<String> {
    parse_source_compatibility(content)
        .or_else(|| parse_target_compatibility(content))
        .or_else(|| parse_language_version_assignment(content))
        .or_else(|| parse_language_version_setter(content))
}

fn parse_source_compatibility(content: &str) -> Option<String> {
    parse_property_assignment(
        content,
        "sourceCompatibility",
        "JavaVersion.VERSION_",
        |remainder| {
            let trimmed = remainder.trim_start();
            match trimmed.chars().next() {
                Some(c) if c.is_ascii_alphanumeric() || c == '_' => None,
                _ => Some(trimmed),
            }
        },
    )
}

fn parse_target_compatibility(content: &str) -> Option<String> {
    parse_property_assignment(
        content,
        "targetCompatibility",
        "JavaVersion.VERSION_",
        |remainder| {
            let trimmed = remainder.trim_start();
            match trimmed.chars().next() {
                Some(c) if c.is_ascii_alphanumeric() || c == '_' => None,
                _ => Some(trimmed),
            }
        },
    )
}

fn parse_language_version_assignment(content: &str) -> Option<String> {
    parse_property_assignment(
        content,
        "languageVersion",
        "JavaLanguageVersion.of(",
        |remainder| remainder.trim_start().strip_prefix(')'),
    )
}

fn parse_language_version_setter(content: &str) -> Option<String> {
    parse_setter(
        content,
        "languageVersion.set(",
        "JavaLanguageVersion.of(",
        |remainder| {
            let trimmed = remainder.trim_start();
            let after_first = trimmed.strip_prefix(')')?;
            let after_second = after_first.trim_start().strip_prefix(')')?;
            Some(after_second)
        },
    )
}

fn parse_property_assignment<F>(
    content: &str,
    property: &str,
    value_prefix: &str,
    suffix_ok: F,
) -> Option<String>
where
    F: Fn(&str) -> Option<&str>,
{
    for line in content.lines() {
        if let Some(value) = match_property_line(line, property, value_prefix, &suffix_ok) {
            return Some(value);
        }
    }
    None
}

fn match_property_line<F>(
    line: &str,
    property: &str,
    value_prefix: &str,
    suffix_ok: &F,
) -> Option<String>
where
    F: Fn(&str) -> Option<&str>,
{
    let trimmed = line.trim_start();
    if !trimmed.starts_with(property) {
        return None;
    }
    let remainder = &trimmed[property.len()..];
    if !property_boundary(remainder) {
        return None;
    }
    let after_property = remainder.trim_start();
    if !after_property.starts_with('=') {
        return None;
    }
    let after_eq = after_property[1..].trim_start();
    if !after_eq.starts_with(value_prefix) {
        return None;
    }
    let remaining = &after_eq[value_prefix.len()..];
    let digits = digits_prefix(remaining);
    if digits.is_empty() {
        return None;
    }
    let remainder_after_digits = &remaining[digits.len()..];
    if let Some(after_suffix) = suffix_ok(remainder_after_digits)
        && line_end_clean(after_suffix)
    {
        return Some(digits.to_string());
    }
    None
}

fn parse_setter<F>(content: &str, setter: &str, value_prefix: &str, suffix_ok: F) -> Option<String>
where
    F: Fn(&str) -> Option<&str>,
{
    for line in content.lines() {
        if let Some(value) = match_setter_line(line, setter, value_prefix, &suffix_ok) {
            return Some(value);
        }
    }
    None
}

fn match_setter_line<F>(
    line: &str,
    setter: &str,
    value_prefix: &str,
    suffix_ok: &F,
) -> Option<String>
where
    F: Fn(&str) -> Option<&str>,
{
    let trimmed = line.trim_start();
    if !trimmed.starts_with(setter) {
        return None;
    }
    let remainder = &trimmed[setter.len()..];
    if !remainder.starts_with(value_prefix) {
        return None;
    }
    let remaining = &remainder[value_prefix.len()..];
    let digits = digits_prefix(remaining);
    if digits.is_empty() {
        return None;
    }
    let remainder_after_digits = &remaining[digits.len()..];
    if let Some(after_suffix) = suffix_ok(remainder_after_digits)
        && line_end_clean(after_suffix)
    {
        return Some(digits.to_string());
    }
    None
}

fn property_boundary(remainder: &str) -> bool {
    remainder
        .chars()
        .next()
        .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_')
}

fn line_end_clean(remaining: &str) -> bool {
    let mut trimmed = remaining.trim_start();
    if let Some(rest) = trimmed.strip_prefix(';') {
        trimmed = rest;
    }
    trimmed = trimmed.trim_start();
    trimmed.is_empty() || trimmed.starts_with("//")
}

fn digits_prefix(text: &str) -> &str {
    let mut len = 0;
    for b in text.as_bytes() {
        if b.is_ascii_digit() {
            len += 1;
        } else {
            break;
        }
    }
    &text[..len]
}
