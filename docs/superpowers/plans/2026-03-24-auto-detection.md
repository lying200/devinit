# Auto Detection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a first project auto-detection flow that selects one primary language, infers a small set of safe configuration fields, and falls back cleanly to the existing manual flow when detection is unavailable or rejected.

**Architecture:** Introduce a dedicated detection layer between the existing preflight checks and manual prompting. The detection engine runs per-language detectors, normalizes the winning result into the existing `Language` enum, and hands that result to a high-level language-resolution flow in `src/main.rs` without changing rendering or Git ignore behavior.

**Tech Stack:** Rust, clap, dialoguer, serde, tera

---

## File Structure

- Create: `src/detection.rs`
  Public detection entry point and top-level re-exports.
- Create: `src/detection/types.rs`
  Detection result types such as `DetectionOutcome`, `LanguageCandidate`, and `DetectionConfidence`.
- Create: `src/detection/engine.rs`
  Detector orchestration and primary-language selection.
- Create: `src/detection/detectors.rs`
  Detector module declarations and detector list helpers.
- Create: `src/detection/detectors/rust.rs`
  Rust project detection.
- Create: `src/detection/detectors/python.rs`
  Python project detection.
- Create: `src/detection/detectors/go.rs`
  Go project detection.
- Create: `src/detection/detectors/java.rs`
  Java project detection.
- Create: `src/detection/detectors/javascript.rs`
  JavaScript project detection.
- Modify: `src/lib.rs`
  Export the new detection module.
- Modify: `src/main.rs`
  Replace inline language-resolution logic with explicit-language override plus detection/manual fallback.
- Modify: `src/prompt.rs`
  Add concise confirmation prompt helpers for detected config and extract manual language selection flow if needed.
- Create: `tests/detection_engine_tests.rs`
  Engine-level tests for candidate aggregation and priority selection.
- Create: `tests/detection_cli_tests.rs`
  CLI-flow tests around explicit `--lang`, detection acceptance, and detection fallback.
- Create: `tests/detection_rust_tests.rs`
  Rust detector coverage.
- Create: `tests/detection_python_tests.rs`
  Python detector coverage.
- Create: `tests/detection_go_tests.rs`
  Go detector coverage.
- Create: `tests/detection_java_tests.rs`
  Java detector coverage.
- Create: `tests/detection_javascript_tests.rs`
  JavaScript detector coverage.

## Task 1: Add Detection Result Types

**Files:**
- Create: `src/detection.rs`
- Create: `src/detection/types.rs`
- Modify: `src/lib.rs`
- Test: `tests/detection_engine_tests.rs`

- [ ] **Step 1: Write the failing type-shape test**

Add a test in `tests/detection_engine_tests.rs` that constructs:

```rust
use devinit::detection::{DetectionConfidence, DetectionOutcome, LanguageCandidate};
use devinit::schema::Language;

#[test]
fn detection_types_support_single_language_match() {
    let candidate = LanguageCandidate {
        language: Language::Rust {
            channel: None,
            version: Some("1.76.0".to_string()),
            components: None,
            targets: None,
        },
        confidence: DetectionConfidence::High,
        reasons: vec!["found Cargo.toml".to_string()],
    };

    let outcome = DetectionOutcome::Match { candidate };

    match outcome {
        DetectionOutcome::Match { candidate } => {
            assert_eq!(candidate.confidence, DetectionConfidence::High);
            assert_eq!(candidate.reasons, vec!["found Cargo.toml"]);
        }
        DetectionOutcome::NoMatch => panic!("expected match"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_engine_tests`
Expected: FAIL because the `detection` module and types do not exist.

- [ ] **Step 3: Write minimal implementation**

Create:

- `src/detection.rs` with:

```rust
pub mod engine;
pub mod detectors;
pub mod types;

pub use engine::detect_project;
pub use types::{DetectionConfidence, DetectionOutcome, LanguageCandidate};
```

- `src/detection/types.rs` with:

```rust
use crate::schema::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageCandidate {
    pub language: Language,
    pub confidence: DetectionConfidence,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionOutcome {
    NoMatch,
    Match { candidate: LanguageCandidate },
}
```

Update `src/lib.rs` to export:

```rust
pub mod detection;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_engine_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection.rs src/detection/types.rs src/lib.rs tests/detection_engine_tests.rs
git commit -m "feat: add detection result types"
```

## Task 2: Add Engine Skeleton and Primary-Language Selection

**Files:**
- Create: `src/detection/engine.rs`
- Create: `src/detection/detectors.rs`
- Test: `tests/detection_engine_tests.rs`

- [ ] **Step 1: Write the failing engine tests**

Extend `tests/detection_engine_tests.rs` with tests for:

```rust
#[test]
fn select_primary_candidate_returns_no_match_for_empty_list() { /* expect NoMatch */ }

#[test]
fn select_primary_candidate_returns_single_candidate() { /* expect same candidate */ }

#[test]
fn select_primary_candidate_prefers_rust_over_javascript() { /* expect Rust */ }
```

Use `LanguageCandidate` values with `Language::Rust` and `Language::JavaScript`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_engine_tests`
Expected: FAIL because selection helpers and engine entry points do not exist.

- [ ] **Step 3: Write minimal implementation**

Create `src/detection/engine.rs` with:

```rust
use std::{io, path::Path};

use super::detectors::run_detectors;
use super::types::{DetectionOutcome, LanguageCandidate};
use crate::schema::Language;

pub fn detect_project(target_dir: &Path) -> io::Result<DetectionOutcome> {
    let candidates = run_detectors(target_dir)?;
    Ok(select_primary_candidate(candidates))
}

pub fn select_primary_candidate(candidates: Vec<LanguageCandidate>) -> DetectionOutcome {
    let mut candidates = candidates;
    candidates.sort_by_key(priority_key);
    match candidates.into_iter().next() {
        Some(candidate) => DetectionOutcome::Match { candidate },
        None => DetectionOutcome::NoMatch,
    }
}

fn priority_key(candidate: &LanguageCandidate) -> usize {
    match candidate.language {
        Language::Rust { .. } => 0,
        Language::Python { .. } => 1,
        Language::Go { .. } => 2,
        Language::Java { .. } => 3,
        Language::JavaScript { .. } => 4,
    }
}
```

Create `src/detection/detectors.rs` with:

```rust
use std::{io, path::Path};

use super::types::LanguageCandidate;

pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

pub fn run_detectors(_target_dir: &Path) -> io::Result<Vec<LanguageCandidate>> {
    Ok(Vec::new())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_engine_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/engine.rs src/detection/detectors.rs tests/detection_engine_tests.rs
git commit -m "feat: add detection engine skeleton"
```

## Task 3: Implement Rust Detection

**Files:**
- Create: `src/detection/detectors/rust.rs`
- Modify: `src/detection/detectors.rs`
- Test: `tests/detection_rust_tests.rs`

- [ ] **Step 1: Write the failing Rust detector tests**

Create `tests/detection_rust_tests.rs` with tests for:

```rust
#[test]
fn rust_detector_returns_none_without_cargo_toml() { /* expect None */ }

#[test]
fn rust_detector_detects_rust_from_cargo_toml() { /* expect Language::Rust */ }

#[test]
fn rust_detector_reads_version_from_rust_toolchain_toml() { /* expect version or channel set */ }
```

Use temporary directories and small fixture files:

- `Cargo.toml`
- `rust-toolchain.toml` containing a simple toolchain value

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_rust_tests`
Expected: FAIL because the Rust detector does not exist.

- [ ] **Step 3: Write minimal implementation**

Implement a detector function like:

```rust
pub fn detect(target_dir: &Path) -> io::Result<Option<LanguageCandidate>>
```

Behavior:

- if `Cargo.toml` does not exist, return `Ok(None)`
- if `Cargo.toml` exists, return `Language::Rust`
- if `rust-toolchain.toml` or `rust-toolchain` provides a simple explicit value, set either `version` or `channel`
- set `confidence` to `High`
- include reasons such as `found Cargo.toml`

Wire it into `run_detectors(...)`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_rust_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/detectors/rust.rs src/detection/detectors.rs tests/detection_rust_tests.rs
git commit -m "feat: add rust project detection"
```

## Task 4: Implement Python Detection

**Files:**
- Create: `src/detection/detectors/python.rs`
- Modify: `src/detection/detectors.rs`
- Test: `tests/detection_python_tests.rs`

- [ ] **Step 1: Write the failing Python detector tests**

Create `tests/detection_python_tests.rs` with tests for:

```rust
#[test]
fn python_detector_returns_none_without_python_signals() { /* expect None */ }

#[test]
fn python_detector_detects_python_from_pyproject_toml() { /* expect Language::Python */ }

#[test]
fn python_detector_reads_version_from_dot_python_version() { /* expect version set */ }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_python_tests`
Expected: FAIL because the Python detector does not exist.

- [ ] **Step 3: Write minimal implementation**

Implement Python detection with:

- `pyproject.toml` or `requirements.txt` as a project signal
- `.python-version` as a simple version source
- confidence `High` for `pyproject.toml`, `Medium` for `requirements.txt`
- reasons describing the matched files

Wire it into `run_detectors(...)`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_python_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/detectors/python.rs src/detection/detectors.rs tests/detection_python_tests.rs
git commit -m "feat: add python project detection"
```

## Task 5: Implement Go Detection

**Files:**
- Create: `src/detection/detectors/go.rs`
- Modify: `src/detection/detectors.rs`
- Test: `tests/detection_go_tests.rs`

- [ ] **Step 1: Write the failing Go detector tests**

Create `tests/detection_go_tests.rs` with tests for:

```rust
#[test]
fn go_detector_returns_none_without_go_mod() { /* expect None */ }

#[test]
fn go_detector_detects_go_from_go_mod() { /* expect Language::Go */ }

#[test]
fn go_detector_reads_version_from_go_mod_go_directive() { /* expect version set */ }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_go_tests`
Expected: FAIL because the Go detector does not exist.

- [ ] **Step 3: Write minimal implementation**

Implement Go detection:

- `go.mod` is the project signal
- parse a simple `go 1.xx` line for version
- confidence `High`
- include reasons such as `found go.mod`

Wire it into `run_detectors(...)`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_go_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/detectors/go.rs src/detection/detectors.rs tests/detection_go_tests.rs
git commit -m "feat: add go project detection"
```

## Task 6: Implement Java Detection

**Files:**
- Create: `src/detection/detectors/java.rs`
- Modify: `src/detection/detectors.rs`
- Test: `tests/detection_java_tests.rs`

- [ ] **Step 1: Write the failing Java detector tests**

Create `tests/detection_java_tests.rs` with tests for:

```rust
#[test]
fn java_detector_returns_none_without_java_signals() { /* expect None */ }

#[test]
fn java_detector_detects_java_from_pom_xml() { /* expect maven_enable = Some(true) */ }

#[test]
fn java_detector_detects_java_from_gradle_file() { /* expect gradle_enable = Some(true) */ }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_java_tests`
Expected: FAIL because the Java detector does not exist.

- [ ] **Step 3: Write minimal implementation**

Implement Java detection:

- `pom.xml`, `build.gradle`, and `build.gradle.kts` are signals
- `pom.xml` sets `maven_enable = Some(true)`
- Gradle files set `gradle_enable = Some(true)`
- confidence `High`
- if both Maven and Gradle files exist, set both booleans

Wire it into `run_detectors(...)`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_java_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/detectors/java.rs src/detection/detectors.rs tests/detection_java_tests.rs
git commit -m "feat: add java project detection"
```

## Task 7: Implement JavaScript Detection

**Files:**
- Create: `src/detection/detectors/javascript.rs`
- Modify: `src/detection/detectors.rs`
- Test: `tests/detection_javascript_tests.rs`

- [ ] **Step 1: Write the failing JavaScript detector tests**

Create `tests/detection_javascript_tests.rs` with tests for:

```rust
#[test]
fn javascript_detector_returns_none_without_package_json() { /* expect None */ }

#[test]
fn javascript_detector_detects_javascript_from_package_json() { /* expect Language::JavaScript */ }

#[test]
fn javascript_detector_reads_package_manager_from_package_json() { /* expect package_manager set */ }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_javascript_tests`
Expected: FAIL because the JavaScript detector does not exist.

- [ ] **Step 3: Write minimal implementation**

Implement JavaScript detection:

- `package.json` is the project signal
- parse a simple `"packageManager": "pnpm@9.0.0"` style string
- map the manager name to `package_manager`
- confidence `High`
- include reasons such as `found package.json`

Leave ambiguous `engines.node` handling for a later step unless a test explicitly requires a simple supported format.

Wire it into `run_detectors(...)`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_javascript_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/detectors/javascript.rs src/detection/detectors.rs tests/detection_javascript_tests.rs
git commit -m "feat: add javascript project detection"
```

## Task 8: Wire Engine to Run All Detectors

**Files:**
- Modify: `src/detection/detectors.rs`
- Test: `tests/detection_engine_tests.rs`

- [ ] **Step 1: Write the failing aggregation tests**

Extend `tests/detection_engine_tests.rs` with a filesystem-backed test:

```rust
#[test]
fn detect_project_returns_rust_match_for_cargo_project() { /* expect DetectionOutcome::Match */ }
```

Use a temp directory with `Cargo.toml`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_engine_tests`
Expected: FAIL because `run_detectors(...)` still returns an empty list.

- [ ] **Step 3: Write minimal implementation**

Update `run_detectors(...)` to:

- call each detector in a fixed order
- push any returned candidates into a `Vec<LanguageCandidate>`
- return the collected candidates

Example shape:

```rust
if let Some(candidate) = rust::detect(target_dir)? {
    candidates.push(candidate);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_engine_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/detection/detectors.rs tests/detection_engine_tests.rs
git commit -m "feat: wire detection engine to language detectors"
```

## Task 9: Add Detection Confirmation Prompt

**Files:**
- Modify: `src/prompt.rs`
- Test: `tests/detection_cli_tests.rs`

- [ ] **Step 1: Write the failing confirmation-shape tests**

Create `tests/detection_cli_tests.rs` with a pure test for a summary formatter helper, for example:

```rust
#[test]
fn detected_summary_includes_language_and_reasons() {
    let summary = format_detected_summary(&candidate);
    assert!(summary.contains("Rust"));
    assert!(summary.contains("found Cargo.toml"));
}
```

This should avoid trying to drive a real terminal prompt in the first red-green cycle.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_cli_tests`
Expected: FAIL because the summary helper does not exist.

- [ ] **Step 3: Write minimal implementation**

In `src/prompt.rs`, add:

- a helper to format the detected summary for display
- a helper to confirm the detected candidate using `dialoguer::Confirm`

Suggested signatures:

```rust
pub fn format_detected_summary(candidate: &LanguageCandidate) -> String
pub fn confirm_detected_config(candidate: &LanguageCandidate) -> bool
```

The formatter should include:

- detected language
- non-empty key fields when present
- reasons

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_cli_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/prompt.rs tests/detection_cli_tests.rs
git commit -m "feat: add detected config confirmation helpers"
```

## Task 10: Refactor Main Flow to Resolve Language Config

**Files:**
- Modify: `src/main.rs`
- Modify: `src/prompt.rs`
- Test: `tests/detection_cli_tests.rs`

- [ ] **Step 1: Write the failing CLI-resolution tests**

Add integration-style tests that exercise the non-interactive edges:

```rust
#[test]
fn explicit_lang_skips_detection() { /* expect generation succeeds without detection files */ }

#[test]
fn missing_detection_falls_back_to_manual_flow_path() { /* assert detection no-match path is reachable */ }
```

For the first version, keep these tests focused on branching helpers rather than full TTY prompts if necessary.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test detection_cli_tests`
Expected: FAIL because `src/main.rs` does not yet resolve explicit/manual/detected flows through a shared helper.

- [ ] **Step 3: Write minimal implementation**

Refactor `src/main.rs` to introduce a helper such as:

```rust
fn resolve_language_config(target_dir: &Path, cli_lang: Option<LanguageChoice>) -> io::Result<Language>
```

Behavior:

- if `cli_lang` is `Some`, use the existing explicit manual prompt function for that language
- if `cli_lang` is `None`, call `detect_project(target_dir)?`
- if `NoMatch`, use the existing manual flow
- if `Match`, call the new confirmation helper
- if confirmed, return the detected `Language`
- if rejected, use the existing manual flow

Keep file generation and Git ignore logic unchanged.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test detection_cli_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/prompt.rs tests/detection_cli_tests.rs
git commit -m "feat: resolve language config through auto detection"
```

## Task 11: Run Full Verification

**Files:**
- Modify: any files needed to fix regressions

- [ ] **Step 1: Run detector tests**

Run:

```bash
cargo test --test detection_engine_tests
cargo test --test detection_rust_tests
cargo test --test detection_python_tests
cargo test --test detection_go_tests
cargo test --test detection_java_tests
cargo test --test detection_javascript_tests
cargo test --test detection_cli_tests
```

Expected: all PASS

- [ ] **Step 2: Run full project tests**

Run:

```bash
cargo test
```

Expected: all PASS

- [ ] **Step 3: Run clippy**

Run:

```bash
cargo clippy --all-targets --all-features
```

Expected: no new warnings introduced by the detection implementation

- [ ] **Step 4: Commit final cleanup if needed**

```bash
git add .
git commit -m "test: verify auto detection flow"
```
