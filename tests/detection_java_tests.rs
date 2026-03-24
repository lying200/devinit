use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::detection::{DetectionConfidence, LanguageCandidate};
use devinit::detection::detectors::java::detect;
use devinit::schema::Language;

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("devinit-java-detect-{name}-{pid}-{nanos}"))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn java_detector_returns_none_without_java_signals() {
    let dir = unique_test_dir("no-signals");
    create_dir(&dir);

    let result = detect(&dir).unwrap();

    assert_eq!(result, None);
}

#[test]
fn java_detector_detects_java_from_pom_xml() {
    let dir = unique_test_dir("maven");
    create_dir(&dir);
    fs::write(dir.join("pom.xml"), "<project/>\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Java {
                jdk_package: None,
                gradle_enable: None,
                maven_enable: Some(true),
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found pom.xml".to_string()],
        })
    );
}

#[test]
fn java_detector_detects_java_from_gradle_file() {
    let dir = unique_test_dir("gradle");
    create_dir(&dir);
    fs::write(dir.join("build.gradle"), "plugins {}\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Java {
                jdk_package: None,
                gradle_enable: Some(true),
                maven_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found build.gradle".to_string()],
        })
    );
}

#[test]
fn java_detector_reads_jdk_from_maven_release() {
    let dir = unique_test_dir("maven-release");
    create_dir(&dir);
    let pom = r#"<project><properties><maven.compiler.release>21</maven.compiler.release></properties></project>"#;
    fs::write(dir.join("pom.xml"), pom).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since pom exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: Some("pkgs.jdk21".to_string()),
            gradle_enable: None,
            maven_enable: Some(true),
        }
    );
    assert_eq!(candidate.confidence, DetectionConfidence::High);
    assert_eq!(candidate.reasons, vec!["found pom.xml".to_string()]);
}

#[test]
fn java_detector_reads_jdk_from_maven_java_version() {
    let dir = unique_test_dir("maven-java-version");
    create_dir(&dir);
    let pom = r#"<project><properties><java.version>17</java.version></properties></project>"#;
    fs::write(dir.join("pom.xml"), pom).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since pom exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: Some("pkgs.jdk17".to_string()),
            gradle_enable: None,
            maven_enable: Some(true),
        }
    );
}

#[test]
fn java_detector_prefers_first_direct_maven_tag() {
    let dir = unique_test_dir("maven-multiple-tags");
    create_dir(&dir);
    let pom = r#"<project><properties><java.version>${foo}</java.version><java.version>21</java.version></properties></project>"#;
    fs::write(dir.join("pom.xml"), pom).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since pom exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: Some("pkgs.jdk21".to_string()),
            gradle_enable: None,
            maven_enable: Some(true),
        }
    );
}

#[test]
fn java_detector_ignores_commented_maven_versions() {
    let dir = unique_test_dir("maven-commented");
    create_dir(&dir);
    let pom = r#"<project><properties><!-- <java.version>21</java.version> --></properties></project>"#;
    fs::write(dir.join("pom.xml"), pom).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since pom exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: Some(true),
        }
    );
}

#[test]
fn java_detector_ignores_maven_interpolated_versions() {
    let dir = unique_test_dir("maven-interpolated");
    create_dir(&dir);
    let pom = r#"<project><properties><maven.compiler.release>${jdk.version}</maven.compiler.release></properties></project>"#;
    fs::write(dir.join("pom.xml"), pom).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since pom exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: Some(true),
        }
    );
}

#[test]
fn java_detector_reads_jdk_from_gradle_source_compatibility() {
    let dir = unique_test_dir("gradle-source-compat");
    create_dir(&dir);
    let gradle = r#"
plugins {
    java
}

java {
    sourceCompatibility = JavaVersion.VERSION_21
}
"#;
    fs::write(dir.join("build.gradle"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: Some("pkgs.jdk21".to_string()),
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_reads_jdk_from_gradle_target_compatibility() {
    let dir = unique_test_dir("gradle-target-compat");
    create_dir(&dir);
    let gradle = r#"
plugins {
    java
}

java {
    targetCompatibility = JavaVersion.VERSION_21
}
"#;
    fs::write(dir.join("build.gradle"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: Some("pkgs.jdk21".to_string()),
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_ignores_source_target_substrings() {
    let dir = unique_test_dir("gradle-substring");
    create_dir(&dir);
    let gradle = r#"
java {
    sourceCompatibilityExtra = JavaVersion.VERSION_21
    targetCompatibilityExtra = JavaVersion.VERSION_17
}
"#;
    fs::write(dir.join("build.gradle"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_ignores_source_target_augmented_assignment() {
    let dir = unique_test_dir("gradle-augmented");
    create_dir(&dir);
    let gradle = r#"
java {
    sourceCompatibility += JavaVersion.VERSION_21
    targetCompatibility += JavaVersion.VERSION_17
}
"#;
    fs::write(dir.join("build.gradle"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_reads_jdk_from_gradle_language_version() {
    let dir = unique_test_dir("gradle-language-version");
    create_dir(&dir);
    let gradle = r#"
java {
    languageVersion = JavaLanguageVersion.of(17)
}
"#;
    fs::write(dir.join("build.gradle"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: Some("pkgs.jdk17".to_string()),
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_reads_jdk_from_gradle_kotlin_language_version() {
    let dir = unique_test_dir("gradle-kotlin");
    create_dir(&dir);
    let gradle = r#"
java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}
"#;
    fs::write(dir.join("build.gradle.kts"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since kotlin build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: Some("pkgs.jdk21".to_string()),
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_does_not_detect_ambiguous_gradle_versions() {
    let dir = unique_test_dir("gradle-ambiguous");
    create_dir(&dir);
    let gradle = r#"
java {
    sourceCompatibility = JavaVersion.VERSION_${project.version}
}
"#;
    fs::write(dir.join("build.gradle"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_ignores_gradle_variable_indirection() {
    let dir = unique_test_dir("gradle-variable");
    create_dir(&dir);
    let gradle = r#"
def v = JavaVersion.VERSION_21
java {
    sourceCompatibility = v
}
"#;
    fs::write(dir.join("build.gradle"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_ignores_gradle_patterns_in_comments_and_strings() {
    let dir = unique_test_dir("gradle-comments-strings");
    create_dir(&dir);
    let gradle = r#"
// sourceCompatibility = JavaVersion.VERSION_21
println("languageVersion = JavaLanguageVersion.of(21)")
java {
    // targetCompatibility = JavaVersion.VERSION_17
    println("targetCompatibility = JavaVersion.VERSION_17")
}
"#;
    fs::write(dir.join("build.gradle"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_ignores_gradle_computed_expressions() {
    let dir = unique_test_dir("gradle-computed");
    create_dir(&dir);
    let gradle = r#"
java {
    sourceCompatibility = JavaVersion.VERSION_21 + foo
    languageVersion = JavaLanguageVersion.of(21) + foo
    languageVersion.set(JavaLanguageVersion.of(21) + foo)
}
"#;
    fs::write(dir.join("build.gradle"), gradle).unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
}
