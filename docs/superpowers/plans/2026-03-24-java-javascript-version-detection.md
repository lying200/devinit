# Java And JavaScript Version Detection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add conservative Java and JavaScript version detection so auto-detected projects can infer `jdk_package` and Node.js `package` when the project files provide a clear simple version signal.

**Architecture:** Keep all behavior local to the existing Java and JavaScript detectors. Add small parsing helpers that extract simple project-local version signals and map them directly onto the existing `Language` enum fields without changing the engine, resolution flow, or rendering architecture.

**Tech Stack:** Rust, existing detector modules, cargo test, cargo clippy

---

## File Structure

- Modify: `src/detection/detectors/java.rs`
  Add Maven and Gradle version parsing helpers and map Java major versions to `jdk_package`.
- Modify: `src/detection/detectors/javascript.rs`
  Add `engines.node` parsing helpers and map Node major versions to `package`.
- Modify: `tests/detection_java_tests.rs`
  Extend detector tests to cover Java version extraction and unsupported cases.
- Modify: `tests/detection_javascript_tests.rs`
  Extend detector tests to cover Node version extraction and unsupported cases.

## Task 1: Add Java Maven Version Detection

**Files:**
- Modify: `src/detection/detectors/java.rs`
- Modify: `tests/detection_java_tests.rs`

- [ ] **Step 1: Write the failing Maven version tests**

Add tests to `tests/detection_java_tests.rs` for:

```rust
#[test]
fn java_detector_reads_jdk_package_from_maven_compiler_release() { /* expect pkgs.jdk21 */ }

#[test]
fn java_detector_reads_jdk_package_from_java_version_property() { /* expect pkgs.jdk17 */ }

#[test]
fn java_detector_ignores_maven_property_interpolation() { /* expect jdk_package = None */ }
```

Use fixture contents like:

```xml
<project>
  <properties>
    <maven.compiler.release>21</maven.compiler.release>
  </properties>
</project>
```

and:

```xml
<project>
  <properties>
    <java.version>${some.other.property}</java.version>
  </properties>
</project>
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_java_tests`
Expected: FAIL because the Java detector does not yet set `jdk_package`.

- [ ] **Step 3: Write minimal implementation**

In `src/detection/detectors/java.rs`, add helpers like:

```rust
fn detect_jdk_package(target_dir: &Path) -> io::Result<Option<String>>
fn parse_maven_java_version(content: &str) -> Option<String>
fn jdk_package_for_major(major: &str) -> Option<String>
```

Implementation rules:

- look for direct simple values in:
  - `<maven.compiler.release>`
  - `<maven.compiler.source>`
  - `<java.version>`
- reject values containing `${...}`
- extract only simple numeric major versions
- map `21` to `pkgs.jdk21`, `17` to `pkgs.jdk17`, and generally `pkgs.jdk<major>`

Set `jdk_package` when a supported value is found, keeping `maven_enable` behavior unchanged.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_java_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/detectors/java.rs tests/detection_java_tests.rs
git commit -m "feat: detect java version from maven files"
```

## Task 2: Add Java Gradle Version Detection

**Files:**
- Modify: `src/detection/detectors/java.rs`
- Modify: `tests/detection_java_tests.rs`

- [ ] **Step 1: Write the failing Gradle version tests**

Extend `tests/detection_java_tests.rs` with:

```rust
#[test]
fn java_detector_reads_jdk_package_from_gradle_source_compatibility() { /* expect pkgs.jdk21 */ }

#[test]
fn java_detector_reads_jdk_package_from_gradle_toolchain_language_version() { /* expect pkgs.jdk17 */ }

#[test]
fn java_detector_reads_jdk_package_from_gradle_kts_toolchain_language_version() { /* expect pkgs.jdk21 */ }

#[test]
fn java_detector_ignores_unsupported_gradle_expressions() { /* expect jdk_package = None */ }
```

Use simple fixtures like:

```gradle
sourceCompatibility = JavaVersion.VERSION_21
```

and:

```gradle
java {
  toolchain {
    languageVersion = JavaLanguageVersion.of(17)
  }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_java_tests`
Expected: FAIL because Gradle version extraction is not implemented yet.

- [ ] **Step 3: Write minimal implementation**

Add helpers such as:

```rust
fn parse_gradle_java_version(content: &str) -> Option<String>
```

Support only simple direct patterns:

- `JavaVersion.VERSION_21`
- `JavaLanguageVersion.of(21)`
- Kotlin DSL equivalents with the same literal values

Do not support:

- variables
- computed expressions
- plugin-generated logic

Use the same `jdk_package_for_major(...)` mapping as Maven.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_java_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/detectors/java.rs tests/detection_java_tests.rs
git commit -m "feat: detect java version from gradle files"
```

## Task 3: Add JavaScript `engines.node` Version Detection

**Files:**
- Modify: `src/detection/detectors/javascript.rs`
- Modify: `tests/detection_javascript_tests.rs`

- [ ] **Step 1: Write the failing JavaScript version tests**

Extend `tests/detection_javascript_tests.rs` with:

```rust
#[test]
fn javascript_detector_reads_node_package_from_engines_node_major() { /* expect pkgs.nodejs_20 */ }

#[test]
fn javascript_detector_reads_node_package_from_caret_range() { /* expect pkgs.nodejs_20 */ }

#[test]
fn javascript_detector_reads_node_package_from_single_major_bounded_range() { /* expect pkgs.nodejs_20 */ }

#[test]
fn javascript_detector_ignores_ambiguous_node_ranges() { /* expect package = None */ }
```

Use fixture contents like:

```json
{
  "name": "demo",
  "engines": {
    "node": "20"
  }
}
```

and:

```json
{
  "name": "demo",
  "engines": {
    "node": ">=18"
  }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_javascript_tests`
Expected: FAIL because the JavaScript detector does not yet populate `package`.

- [ ] **Step 3: Write minimal implementation**

In `src/detection/detectors/javascript.rs`, add helpers like:

```rust
fn parse_engines_node_major(content: &str) -> Option<String>
fn node_package_for_major(major: &str) -> Option<String>
```

Support only simple cases:

- `20`
- `20.11.1`
- `v20`
- `^20`
- `~20.11`
- `>=20`
- `>=20 <21`

Reject ambiguous cases such as:

- `>=18`
- `18 || 20`
- `lts/*`
- `*`

Map a detected major version to `pkgs.nodejs_<major>`.

Keep `packageManager` behavior unchanged.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_javascript_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/detectors/javascript.rs tests/detection_javascript_tests.rs
git commit -m "feat: detect node version from package json"
```

## Task 4: Run Focused Verification

**Files:**
- Modify: any files needed to fix regressions

- [ ] **Step 1: Run Java detector tests**

Run:

```bash
cargo test --test detection_java_tests
```

Expected: PASS

- [ ] **Step 2: Run JavaScript detector tests**

Run:

```bash
cargo test --test detection_javascript_tests
```

Expected: PASS

- [ ] **Step 3: Run full test suite**

Run:

```bash
cargo test
```

Expected: PASS

- [ ] **Step 4: Run clippy**

Run:

```bash
cargo clippy --all-targets --all-features
```

Expected: PASS with no new warnings introduced by the version detection logic

- [ ] **Step 5: Commit final cleanup if needed**

```bash
git add .
git commit -m "test: verify java and javascript version detection"
```
