# Java And JavaScript Version Detection Design

## Goal

Extend the new project auto-detection flow so that Java and JavaScript projects can infer a usable runtime version when the project files provide a clear signal.

The first version is intentionally narrow:

- Java detection should infer a JDK package only from simple Maven or Gradle configuration
- JavaScript detection should infer a Node.js package only from simple `package.json` `engines.node` values

This work should improve generated `devenv` usability for these ecosystems without expanding the overall detection architecture.

## Scope

This design supports only:

- Java version detection from `pom.xml`
- Java version detection from `build.gradle`
- Java version detection from `build.gradle.kts`
- JavaScript version detection from `package.json`
- mapping detected Java versions to `Language::Java { jdk_package }`
- mapping detected JavaScript versions to `Language::JavaScript { package }`

This design explicitly does not support:

- `.nvmrc`
- `.node-version`
- Volta config
- asdf config
- reading `java -version`
- reading `node -v`
- parsing complex Gradle build logic
- parsing complex Maven property interpolation
- supporting every possible `engines.node` semver expression

## Why This Scope

The current auto-detection implementation already extracts version-like information for:

- Rust, from `rust-toolchain(.toml)`
- Python, from `.python-version`
- Go, from `go.mod`

That makes Java and JavaScript the largest remaining gaps for version-sensitive project setup.

At the same time, both ecosystems have many possible version sources. If the first implementation tries to support all of them, the detector layer will quickly become brittle and hard to reason about.

The right first step is therefore conservative:

- use project-local manifest files only
- support simple, high-confidence patterns
- skip ambiguous cases

This preserves the detector philosophy already established in the project: prefer under-detection to wrong detection.

## Current Project Structure

The current auto-detection path is already in place:

- `src/detection/detectors/java.rs` detects Java projects and build-tool flags
- `src/detection/detectors/javascript.rs` detects JavaScript projects and package managers
- `src/detection/engine.rs` chooses the primary language
- `src/schema/context.rs` defines the renderable `Language` enum
- `src/main.rs` only consumes the final detected `Language`

That means this feature should stay local to the per-language detectors and should not modify the engine or the main flow.

## Recommended Approach

Keep the existing detector boundaries and add small helper functions inside the Java and JavaScript detectors.

The detectors should continue to return `LanguageCandidate`, but now populate:

- Java: `jdk_package`
- JavaScript: `package`

when and only when a clear version can be extracted.

This keeps the architecture stable:

- no new global types
- no new flow branches in `main.rs`
- no new engine scoring logic

## Java Version Detection

### Supported Sources

The Java detector should inspect these sources in this order:

1. `pom.xml`
2. `build.gradle`
3. `build.gradle.kts`

### Supported Maven Patterns

The first version should support only direct, simple values from:

- `<maven.compiler.release>`
- `<maven.compiler.source>`
- `<java.version>`

Examples that should be supported:

```xml
<maven.compiler.release>21</maven.compiler.release>
<maven.compiler.source>17</maven.compiler.source>
<java.version>21</java.version>
```

Examples that should not be supported in the first version:

```xml
<maven.compiler.release>${java.version}</maven.compiler.release>
<java.version>${some.other.property}</java.version>
```

If the value is not a direct simple version string, the detector should leave `jdk_package` unset.

### Supported Gradle Patterns

The first version should support only a few direct patterns such as:

```gradle
sourceCompatibility = JavaVersion.VERSION_21
targetCompatibility = JavaVersion.VERSION_17
java {
  toolchain {
    languageVersion = JavaLanguageVersion.of(21)
  }
}
```

and Kotlin Gradle equivalents such as:

```kotlin
sourceCompatibility = JavaVersion.VERSION_21
languageVersion.set(JavaLanguageVersion.of(21))
```

The detector should not attempt to execute build logic, resolve variables, or interpret complex expressions.

### Java Mapping Rules

After extracting a simple Java major version, map it to:

- `21` -> `pkgs.jdk21`
- `17` -> `pkgs.jdk17`
- other simple major values -> `pkgs.jdk<major>`

Examples:

- `8` -> `pkgs.jdk8`
- `11` -> `pkgs.jdk11`
- `21` -> `pkgs.jdk21`

If the extracted value is not a clean Java major version, do not populate `jdk_package`.

### Java Detector Output

The existing build-tool detection remains unchanged:

- `pom.xml` still sets `maven_enable = Some(true)`
- Gradle files still set `gradle_enable = Some(true)`

Version detection only adds:

- `jdk_package = Some(...)` when a simple supported version is found

## JavaScript Version Detection

### Supported Source

The JavaScript detector should inspect only:

- `package.json`

### Supported `engines.node` Patterns

The first version should support only simple values that can be reduced to one clear major version.

Examples that should be supported:

- `"20"`
- `"20.11.1"`
- `"v20"`
- `"^20"`
- `"~20.11"`
- `">=20"`
- `">=20 <21"`

Examples that should not be supported:

- `"lts/*"`
- `"*"`
- `">=18"`
- `"18 || 20"`
- any format that implies multiple valid major versions

### JavaScript Mapping Rules

After extracting a single clear Node.js major version, map it to:

- `20` -> `pkgs.nodejs_20`
- `22` -> `pkgs.nodejs_22`
- in general: `pkgs.nodejs_<major>`

If the version expression is ambiguous or unsupported, do not populate `package`.

### JavaScript Detector Output

The existing package-manager detection remains unchanged:

- `packageManager` still populates `package_manager`

Version detection only adds:

- `package = Some(...)` when `engines.node` provides a clear major version

## Detector Behavior Rules

For both languages:

- keep existing project-detection behavior unchanged
- add version/package inference only when the source is explicit and simple
- never guess from machine-global tools
- never guess from ambiguous ranges
- never downgrade confidence just because version is absent

In other words:

- language detection and version detection are related, but version remains optional

## Implementation Boundaries

This feature should stay inside:

- `src/detection/detectors/java.rs`
- `src/detection/detectors/javascript.rs`

Add small helpers such as:

- `parse_maven_java_version(...)`
- `parse_gradle_java_version(...)`
- `jdk_package_for_major(...)`
- `parse_engines_node_major(...)`
- `node_package_for_major(...)`

The following areas should remain unchanged unless needed for tests:

- `src/detection/engine.rs`
- `src/main.rs`
- `src/resolution.rs`

## Testing Strategy

Follow TDD and keep tests detector-local.

### Java tests

Add or extend detector tests for:

- Maven `pom.xml` with simple direct version
- Gradle `build.gradle` with simple direct version
- Kotlin Gradle file with simple direct version
- unsupported or ambiguous Java declarations leave `jdk_package` unset
- build-tool booleans still behave as before

### JavaScript tests

Add or extend detector tests for:

- `engines.node = "20"` maps to `pkgs.nodejs_20`
- `engines.node = "^20"` maps to `pkgs.nodejs_20`
- `engines.node = ">=20 <21"` maps to `pkgs.nodejs_20`
- unsupported or ambiguous expressions leave `package` unset
- `packageManager` detection still behaves as before

## Risks

- Java build files are easy to over-parse; the detector must stay conservative
- semver range parsing for Node.js can easily become too broad if not tightly scoped
- users may expect `.nvmrc` or global toolchains to be considered, but that is intentionally excluded

## Non-Goals

- full semver solving
- parsing all Maven property indirections
- parsing arbitrary Gradle DSL
- adding new UI for version explanation
- changing primary language selection
