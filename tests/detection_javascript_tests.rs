use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::detection::detectors::javascript::detect;
use devinit::detection::{DetectionConfidence, LanguageCandidate};
use devinit::schema::Language;

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("devinit-javascript-detect-{name}-{pid}-{nanos}"))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn javascript_detector_returns_none_without_package_json() {
    let dir = unique_test_dir("no-package-json");
    create_dir(&dir);

    let result = detect(&dir).unwrap();

    assert_eq!(result, None);
}

#[test]
fn javascript_detector_detects_javascript_from_package_json() {
    let dir = unique_test_dir("package-json");
    create_dir(&dir);
    fs::write(dir.join("package.json"), "{\n  \"name\": \"demo\"\n}\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}

#[test]
fn javascript_detector_reads_package_manager_from_package_json() {
    let dir = unique_test_dir("package-manager");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"packageManager\": \"pnpm@9.0.0\"\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: Some("pnpm".to_string()),
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found package.json".to_string(),
                "found packageManager".to_string(),
            ],
        })
    );
}

#[test]
fn javascript_detector_reads_node_package_from_engines_node_major() {
    let dir = unique_test_dir("node-major");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \"20\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: Some("pkgs.nodejs_20".to_string()),
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found package.json".to_string(),
                "found engines.node".to_string(),
            ],
        })
    );
}

#[test]
fn javascript_detector_reads_node_package_from_caret_range() {
    let dir = unique_test_dir("node-caret");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \"^20\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: Some("pkgs.nodejs_20".to_string()),
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found package.json".to_string(),
                "found engines.node".to_string(),
            ],
        })
    );
}

#[test]
fn javascript_detector_reads_node_package_from_version_string() {
    let dir = unique_test_dir("node-version-string");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \"20.11.1\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: Some("pkgs.nodejs_20".to_string()),
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found package.json".to_string(),
                "found engines.node".to_string(),
            ],
        })
    );
}

#[test]
fn javascript_detector_reads_node_package_from_v_prefixed_major() {
    let dir = unique_test_dir("node-v-major");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \"v20\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: Some("pkgs.nodejs_20".to_string()),
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found package.json".to_string(),
                "found engines.node".to_string(),
            ],
        })
    );
}

#[test]
fn javascript_detector_reads_node_package_from_tilde_range() {
    let dir = unique_test_dir("node-tilde");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \"~20.11\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: Some("pkgs.nodejs_20".to_string()),
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found package.json".to_string(),
                "found engines.node".to_string(),
            ],
        })
    );
}

#[test]
fn javascript_detector_reads_node_package_from_single_major_bounded_range() {
    let dir = unique_test_dir("node-bounded");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \">=20 <21\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: Some("pkgs.nodejs_20".to_string()),
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found package.json".to_string(),
                "found engines.node".to_string(),
            ],
        })
    );
}

#[test]
fn javascript_detector_reads_node_package_from_open_ended_major_range() {
    let dir = unique_test_dir("node-open-ended-supported");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \">=20\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: Some("pkgs.nodejs_20".to_string()),
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found package.json".to_string(),
                "found engines.node".to_string(),
            ],
        })
    );
}

#[test]
fn javascript_detector_ignores_ambiguous_open_ended_node_range() {
    let dir = unique_test_dir("node-open-ended");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \">=18\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}

#[test]
fn javascript_detector_ignores_lts_wildcard_node_range() {
    let dir = unique_test_dir("node-lts-wildcard");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \"lts/*\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}

#[test]
fn javascript_detector_ignores_multi_major_node_range() {
    let dir = unique_test_dir("node-multi-major");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \"18 || 20\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}

#[test]
fn javascript_detector_ignores_wildcard_node_range() {
    let dir = unique_test_dir("node-wildcard");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \"*\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}

#[test]
fn javascript_detector_ignores_escaped_engines_text_in_string_value() {
    let dir = unique_test_dir("node-escaped-engines-string");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"description\": \"quoted \\\"engines\\\": { \\\"node\\\": \\\"20\\\" } text\"\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}

#[test]
fn javascript_detector_ignores_nested_engines_object() {
    let dir = unique_test_dir("node-nested-engines");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"config\": {\n    \"engines\": {\n      \"node\": \"20\"\n    }\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}

#[test]
fn javascript_detector_ignores_nested_node_key_inside_engines() {
    let dir = unique_test_dir("node-nested-inside-engines");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"npm\": {\n      \"node\": \"20\"\n    }\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}

#[test]
fn javascript_detector_ignores_non_major_bounded_node_range() {
    let dir = unique_test_dir("node-non-major-bounded");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"engines\": {\n    \"node\": \">=20 <21.5\"\n  }\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}
