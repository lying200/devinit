use std::{fs, io, path::Path};

use crate::detection::{DetectionConfidence, LanguageCandidate};
use crate::schema::Language;

pub fn detect(target_dir: &Path) -> io::Result<Option<LanguageCandidate>> {
    let pyproject = target_dir.join("pyproject.toml");
    let requirements = target_dir.join("requirements.txt");

    let (confidence, mut reasons) = if pyproject.exists() {
        (DetectionConfidence::High, vec!["found pyproject.toml".to_string()])
    } else if requirements.exists() {
        (
            DetectionConfidence::Medium,
            vec!["found requirements.txt".to_string()],
        )
    } else {
        return Ok(None);
    };

    // .python-version 优先（精确版本）
    let python_version_file = target_dir.join(".python-version");
    let version = if python_version_file.exists() {
        reasons.push("found .python-version".to_string());
        let value = fs::read_to_string(python_version_file)?;
        let value = value.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    } else if pyproject.exists() {
        // 从 pyproject.toml 的 requires-python 提取版本
        let content = fs::read_to_string(&pyproject)?;
        let v = parse_requires_python(&content);
        if v.is_some() {
            reasons.push("found requires-python".to_string());
        }
        v
    } else {
        None
    };

    Ok(Some(LanguageCandidate {
        language: Language::Python {
            version,
            package: None,
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        },
        confidence,
        reasons,
    }))
}

/// 从 pyproject.toml 的 requires-python 字段提取最低版本。
///
/// 只处理简单明确的格式：
/// - `requires-python = ">=3.11"`
/// - `requires-python = ">=3.11.0"`
///
/// 复杂范围（含 `,`、`||`、`<` 等）跳过，返回 None。
fn parse_requires_python(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("requires-python") {
            let value = value.trim();
            let value = value.strip_prefix('=')?;
            let value = value.trim().trim_matches('"').trim_matches('\'');

            // 跳过复杂范围
            if value.contains(',') || value.contains("||") || value.contains('<') {
                return None;
            }

            let version = value.strip_prefix(">=").or_else(|| value.strip_prefix("=="))?;
            let version = version.trim();

            if is_version_like(version) {
                return Some(version.to_string());
            }
        }
    }
    None
}

fn is_version_like(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '.')
}
