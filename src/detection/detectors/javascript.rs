use std::{fs, io, path::Path};

use crate::detection::{DetectionConfidence, LanguageCandidate};
use crate::schema::Language;

pub fn detect(target_dir: &Path) -> io::Result<Option<LanguageCandidate>> {
    let package_json = target_dir.join("package.json");
    if !package_json.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(package_json)?;
    let package = parse_node_package(&content);
    let package_manager = parse_package_manager(&content);

    let mut reasons = vec!["found package.json".to_string()];
    if package.is_some() {
        reasons.push("found engines.node".to_string());
    }
    if package_manager.is_some() {
        reasons.push("found packageManager".to_string());
    }

    Ok(Some(LanguageCandidate {
        language: Language::JavaScript {
            package,
            package_manager,
            corepack_enable: None,
        },
        confidence: DetectionConfidence::High,
        reasons,
    }))
}

fn parse_package_manager(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("\"packageManager\":") {
            let value = value.trim().trim_end_matches(',').trim().trim_matches('"');
            let manager = value.split('@').next().unwrap_or("").trim();
            if matches!(manager, "npm" | "pnpm" | "yarn" | "bun") {
                return Some(manager.to_string());
            }
        }
    }
    None
}

fn parse_node_package(content: &str) -> Option<String> {
    let major = parse_engines_node_major(content)?;
    Some(format!("pkgs.nodejs_{major}"))
}

fn parse_engines_node_major(content: &str) -> Option<String> {
    let engines = extract_top_level_object_field(content, "engines")?;
    let node = extract_top_level_string_field(engines, "node")?;
    parse_node_major_expr(&node)
}

fn parse_node_major_expr(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.contains("||")
        || trimmed == "*"
        || trimmed == "lts/*"
        || trimmed.contains('*')
    {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix("^") {
        return parse_version_like_major(rest);
    }

    if let Some(rest) = trimmed.strip_prefix("~") {
        return parse_version_like_major(rest);
    }

    if let Some(rest) = trimmed.strip_prefix(">=") {
        return parse_greater_equal_major(rest);
    }

    parse_version_like_major(trimmed)
}

fn parse_greater_equal_major(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if let Some((lower, upper)) = trimmed.split_once('<') {
        let lower_major = parse_major_only(lower.trim())?;
        let upper_major = parse_major_only(upper.trim())?;
        let lower_num: u32 = lower_major.parse().ok()?;
        let upper_num: u32 = upper_major.parse().ok()?;
        if upper_num == lower_num + 1 {
            return Some(lower_major);
        }
        return None;
    }

    let major = parse_major_only(trimmed)?;
    let major_num: u32 = major.parse().ok()?;
    if major_num >= 20 {
        Some(major)
    } else {
        None
    }
}

fn parse_version_like_major(value: &str) -> Option<String> {
    let trimmed = value.trim().strip_prefix('v').unwrap_or(value.trim());
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split('.');
    let major = parts.next()?;
    if !is_ascii_digits(major) {
        return None;
    }

    if parts.any(|part| part.is_empty() || !is_ascii_digits(part)) {
        return None;
    }

    Some(major.to_string())
}

fn parse_major_only(value: &str) -> Option<String> {
    let trimmed = value.trim().strip_prefix('v').unwrap_or(value.trim());
    if is_ascii_digits(trimmed) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn extract_top_level_object_field<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    let value_start = find_top_level_field_value_start(content, key)?;
    let bytes = content.as_bytes();
    if *bytes.get(value_start)? != b'{' {
        return None;
    }

    let end = find_matching_brace(content, value_start)?;
    Some(&content[value_start..=end])
}

fn extract_top_level_string_field(content: &str, key: &str) -> Option<String> {
    let value_start = find_top_level_field_value_start(content, key)?;
    parse_json_string(content, value_start)
}

fn find_top_level_field_value_start(content: &str, key: &str) -> Option<usize> {
    let pattern = format!("\"{key}\"");
    let bytes = content.as_bytes();
    let mut idx = skip_json_whitespace(content, 0);
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    while let Some(byte) = bytes.get(idx) {
        match *byte {
            b'\\' if in_string => escaped = !escaped,
            b'"' if in_string && !escaped => in_string = false,
            b'"' if !in_string => {
                if depth == 1 && content[idx..].starts_with(&pattern) {
                    let mut value_idx = idx + pattern.len();
                    value_idx = skip_json_whitespace(content, value_idx);
                    if content.as_bytes().get(value_idx) == Some(&b':') {
                        value_idx += 1;
                        value_idx = skip_json_whitespace(content, value_idx);
                        return Some(value_idx);
                    }
                }
                in_string = true;
            }
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return None;
                }
            }
            _ => escaped = false,
        }

        if *byte != b'\\' {
            escaped = false;
        }
        idx += 1;
    }

    None
}

fn skip_json_whitespace(content: &str, mut idx: usize) -> usize {
    let bytes = content.as_bytes();
    while let Some(byte) = bytes.get(idx) {
        if byte.is_ascii_whitespace() {
            idx += 1;
        } else {
            break;
        }
    }
    idx
}

fn find_matching_brace(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut idx = start;
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    while let Some(byte) = bytes.get(idx) {
        match *byte {
            b'\\' if in_string => escaped = !escaped,
            b'"' if !escaped => in_string = !in_string,
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => escaped = false,
        }

        if *byte != b'\\' {
            escaped = false;
        }
        idx += 1;
    }

    None
}

fn parse_json_string(content: &str, start: usize) -> Option<String> {
    let bytes = content.as_bytes();
    if *bytes.get(start)? != b'"' {
        return None;
    }

    let mut idx = start + 1;
    let mut value = String::new();
    let mut escaped = false;

    while let Some(byte) = bytes.get(idx) {
        if escaped {
            value.push(*byte as char);
            escaped = false;
            idx += 1;
            continue;
        }

        match *byte {
            b'\\' => escaped = true,
            b'"' => return Some(value),
            _ => value.push(*byte as char),
        }
        idx += 1;
    }

    None
}

fn is_ascii_digits(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_ascii_digit())
}
