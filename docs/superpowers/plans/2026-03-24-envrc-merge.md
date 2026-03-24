# Envrc Merge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let `devinit` create `.envrc` when absent, append a managed block when `.envrc` already exists without one, and skip duplicate insertion on later runs.

**Architecture:** Remove `.envrc` from the hard-stop init guard and route `.envrc` through a dedicated writer in `src/generator.rs`. The dedicated writer will detect a stable `devinit` marker pair, append the rendered block only when needed, and keep other generated files on the existing overwrite path.

**Tech Stack:** Rust, std::fs, clap, tera, cargo test

---

## File Structure

- Modify: `src/init_guard.rs`
  Stop treating `.envrc` as a standalone environment marker.
- Modify: `src/generator.rs`
  Add dedicated `.envrc` merge/write helpers.
- Modify: `templates/.envrc.tera`
  Wrap generated content in stable `devinit` markers.
- Modify: `tests/init_guard_tests.rs`
  Update CLI expectations around existing `.envrc`.
- Modify: `tests/generator_tests.rs`
  Add file-writing tests for create, append, and idempotent skip behavior.

### Task 1: Update Guard Semantics

**Files:**
- Modify: `src/init_guard.rs`
- Modify: `tests/init_guard_tests.rs`
- Test: `tests/init_guard_tests.rs`

- [ ] **Step 1: Write the failing guard tests**

Add or update tests so that:

```rust
#[test]
fn detect_existing_environment_returns_none_for_envrc_only() {
    // create directory with only .envrc
    // expect detect_existing_environment to return None
}

#[test]
fn devinit_does_not_skip_when_only_envrc_exists() {
    // run CLI in a directory containing only .envrc
    // expect stdout to not contain the existing-environment skip message
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test init_guard_tests`
Expected: FAIL because `.envrc` is still treated as a guard marker.

- [ ] **Step 3: Write minimal implementation**

Update `src/init_guard.rs` to remove `.envrc` from `DIRECT_MARKERS`, keeping all other markers unchanged.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test init_guard_tests`
Expected: PASS for the new `.envrc` cases and existing pass for `flake.nix` and related markers.

### Task 2: Add `.envrc` Create/Append/Skip Behavior

**Files:**
- Modify: `src/generator.rs`
- Modify: `templates/.envrc.tera`
- Modify: `tests/generator_tests.rs`
- Test: `tests/generator_tests.rs`

- [ ] **Step 1: Write the failing file-writing tests**

Add tests that:

```rust
#[test]
fn write_files_creates_envrc_when_missing() { /* expect new file with markers */ }

#[test]
fn write_files_appends_devinit_block_to_existing_envrc() { /* expect original content plus one managed block */ }

#[test]
fn write_files_does_not_duplicate_existing_devinit_block() { /* expect unchanged content after second write */ }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test generator_tests`
Expected: FAIL because `write_files` still overwrites `.envrc` and the template has no managed markers.

- [ ] **Step 3: Write minimal implementation**

Implement in `src/generator.rs`:

- constants for the managed start/end markers
- a helper that detects whether existing content already contains both markers
- a helper that appends the rendered block with exactly one blank-line separator when needed
- a branch in `write_files` so `.envrc` uses the dedicated helper and all other files still use `std::fs::write`

Wrap `templates/.envrc.tera` in the same start/end markers used by the helper.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test generator_tests`
Expected: PASS

### Task 3: Run Focused Regression Verification

**Files:**
- Modify: none
- Test: `tests/init_guard_tests.rs`, `tests/generator_tests.rs`

- [ ] **Step 1: Run focused regression tests**

Run: `cargo test --test init_guard_tests --test generator_tests`
Expected: PASS

- [ ] **Step 2: Run the full test suite if focused tests are green**

Run: `cargo test`
Expected: PASS
