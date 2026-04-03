use std::{
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::detection::detectors::java::detect;
use devinit::detection::{DetectionConfidence, LanguageCandidate};
use devinit::schema::Language;

struct TestDir(PathBuf);

impl TestDir {
    fn new(name: &str) -> Self {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("devinit-java-detect-{name}-{pid}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Deref for TestDir {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn java_detector_returns_none_without_java_signals() {
    let dir = TestDir::new("no-signals");


    let result = detect(&dir).unwrap();

    assert_eq!(result, None);
}

#[test]
fn java_detector_detects_java_from_pom_xml() {
    let dir = TestDir::new("maven");

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
    let dir = TestDir::new("gradle");

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
    let dir = TestDir::new("maven-release");

    let pom = r"<project><properties><maven.compiler.release>21</maven.compiler.release></properties></project>";
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
    let dir = TestDir::new("maven-java-version");

    let pom = r"<project><properties><java.version>17</java.version></properties></project>";
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
    let dir = TestDir::new("maven-multiple-tags");

    let pom = r"<project><properties><java.version>${foo}</java.version><java.version>21</java.version></properties></project>";
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
    let dir = TestDir::new("maven-commented");

    let pom =
        r"<project><properties><!-- <java.version>21</java.version> --></properties></project>";
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
    let dir = TestDir::new("maven-interpolated");

    let pom = r"<project><properties><maven.compiler.release>${jdk.version}</maven.compiler.release></properties></project>";
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
    let dir = TestDir::new("gradle-source-compat");

    let gradle = r"
plugins {
    java
}

java {
    sourceCompatibility = JavaVersion.VERSION_21
}
";
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
    let dir = TestDir::new("gradle-target-compat");

    let gradle = r"
plugins {
    java
}

java {
    targetCompatibility = JavaVersion.VERSION_21
}
";
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
    let dir = TestDir::new("gradle-substring");

    let gradle = r"
java {
    sourceCompatibilityExtra = JavaVersion.VERSION_21
    targetCompatibilityExtra = JavaVersion.VERSION_17
}
";
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
    let dir = TestDir::new("gradle-augmented");

    let gradle = r"
java {
    sourceCompatibility += JavaVersion.VERSION_21
    targetCompatibility += JavaVersion.VERSION_17
}
";
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
    let dir = TestDir::new("gradle-language-version");

    let gradle = r"
java {
    languageVersion = JavaLanguageVersion.of(17)
}
";
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
    let dir = TestDir::new("gradle-kotlin");

    let gradle = r"
java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}
";
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
    let dir = TestDir::new("gradle-ambiguous");

    let gradle = r"
java {
    sourceCompatibility = JavaVersion.VERSION_${project.version}
}
";
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
    let dir = TestDir::new("gradle-variable");

    let gradle = r"
def v = JavaVersion.VERSION_21
java {
    sourceCompatibility = v
}
";
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
    let dir = TestDir::new("gradle-comments-strings");

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
    let dir = TestDir::new("gradle-computed");

    let gradle = r"
java {
    sourceCompatibility = JavaVersion.VERSION_21 + foo
    languageVersion = JavaLanguageVersion.of(21) + foo
    languageVersion.set(JavaLanguageVersion.of(21) + foo)
}
";
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
fn java_detector_gradle_wrapper_disables_system_gradle() {
    let dir = TestDir::new("gradle-wrapper");

    fs::write(dir.join("build.gradle"), "plugins {}\n").unwrap();
    fs::write(dir.join("gradlew"), "#!/bin/sh\n").unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: None,
        }
    );
    assert!(candidate.reasons.iter().any(|r| r.contains("gradlew")));
}

#[test]
fn java_detector_both_build_files_prefers_gradle() {
    let dir = TestDir::new("both-build-files");

    fs::write(dir.join("pom.xml"), "<project/>\n").unwrap();
    fs::write(dir.join("build.gradle"), "plugins {}\n").unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build files exist");

    // Gradle takes priority, maven not enabled
    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
    assert!(candidate.reasons.contains(&"found pom.xml".to_string()));
    assert!(candidate
        .reasons
        .contains(&"found build.gradle".to_string()));
}

#[test]
fn java_detector_both_build_files_with_wrapper() {
    let dir = TestDir::new("both-with-wrapper");

    fs::write(dir.join("pom.xml"), "<project/>\n").unwrap();
    fs::write(dir.join("build.gradle"), "plugins {}\n").unwrap();
    fs::write(dir.join("gradlew.bat"), "@echo off\n").unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build files exist");

    // Gradle wrapper present: no system gradle, and gradle takes priority so no maven either
    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: None,
        }
    );
}

#[test]
fn java_detector_gradlew_bat_only_disables_system_gradle() {
    let dir = TestDir::new("gradlew-bat-only");

    fs::write(dir.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(dir.join("gradlew.bat"), "@echo off\n").unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build file exists");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: None,
        }
    );
    assert!(candidate.reasons.iter().any(|r| r.contains("gradlew")));
}

#[test]
fn java_detector_both_gradle_and_gradle_kts() {
    let dir = TestDir::new("gradle-and-kts");

    fs::write(dir.join("build.gradle"), "plugins {}\n").unwrap();
    fs::write(dir.join("build.gradle.kts"), "plugins {}\n").unwrap();

    let candidate = detect(&dir)
        .unwrap()
        .expect("expected candidate since build files exist");

    assert_eq!(
        candidate.language,
        Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
        }
    );
    assert!(candidate.reasons.contains(&"found build.gradle".to_string()));
    assert!(candidate
        .reasons
        .contains(&"found build.gradle.kts".to_string()));
}
