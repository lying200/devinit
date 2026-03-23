# Python Init Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a minimal interactive Python init flow that generates `devenv` Python config for version, package, uv, and venv settings.

**Architecture:** Extend the existing language enum and prompt-based CLI flow instead of introducing a new config layer. Reuse the current Tera template rendering path so Python support follows the same `ProjectContext -> template -> files` pipeline as Rust.

**Tech Stack:** Rust, clap, dialoguer, serde, tera, cargo test

---

### Task 1: Add Failing Python Rendering Tests

**Files:**
- Modify: `tests/generator_tests.rs`
- Test: `tests/generator_tests.rs`

- [ ] **Step 1: Write the failing tests**

Add tests for these cases:

```rust
#[test]
fn test_render_python_base() { /* expect only languages.python.enable = true */ }

#[test]
fn test_render_python_with_version() { /* expect version = "3.11" */ }

#[test]
fn test_render_python_with_package() { /* expect package = pkgs.python311 */ }

#[test]
fn test_render_python_with_uv() { /* expect uv.enable = true */ }

#[test]
fn test_render_python_with_venv() { /* expect venv.enable = true; venv.quiet = true; */ }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test generator_tests`
Expected: FAIL because `Language::Python` does not yet carry the needed fields and templates do not render Python config.

- [ ] **Step 3: Commit**

```bash
git add tests/generator_tests.rs
git commit -m "test: add failing python rendering tests"
```

### Task 2: Add Failing Python File Planning Test

**Files:**
- Modify: `tests/generate_rust_file_tests.rs`
- Test: `tests/generate_rust_file_tests.rs`

- [ ] **Step 1: Write the failing test**

Add a Python planning test that asserts:

- `plan_files` returns 3 files
- filenames include `devenv.nix`, `devenv.yaml`, `.envrc`
- `devenv.nix` contains `languages.python`

- [ ] **Step 2: Run tests to verify it fails**

Run: `cargo test --test generate_rust_file_tests`
Expected: FAIL because Python output is not yet rendered.

- [ ] **Step 3: Commit**

```bash
git add tests/generate_rust_file_tests.rs
git commit -m "test: add failing python planning test"
```

### Task 3: Extend the Python Schema and CLI Entry

**Files:**
- Modify: `src/schema/context.rs`
- Modify: `src/main.rs`
- Modify: `src/prompt.rs`
- Test: `tests/generator_tests.rs`

- [ ] **Step 1: Implement the minimal schema**

Change `Language::Python` to:

```rust
Python {
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uv_enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    venv_enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    venv_quiet: Option<bool>,
}
```

- [ ] **Step 2: Add the Python prompt function**

Implement `prompt_python_config()` in `src/prompt.rs` with:

- default confirmation path
- optional `version`
- optional `package`
- `enable uv?`
- `enable venv?`
- conditional `quiet?`

- [ ] **Step 3: Wire the CLI to the new prompt**

In `src/main.rs`, import `prompt_python_config` and change the Python branch to call it.

- [ ] **Step 4: Run targeted tests**

Run: `cargo test --test generator_tests`
Expected: still FAIL because template rendering is not done yet, but compile-time model errors should be resolved.

- [ ] **Step 5: Commit**

```bash
git add src/schema/context.rs src/prompt.rs src/main.rs
git commit -m "feat: add python init schema and prompts"
```

### Task 4: Render Python Config in the Nix Template

**Files:**
- Modify: `templates/devenv.nix.tera`
- Test: `tests/generator_tests.rs`
- Test: `tests/generate_rust_file_tests.rs`

- [ ] **Step 1: Implement minimal template support**

Add a Python branch that emits:

```nix
languages.python = {
  enable = true;
  version = "3.11";
  package = pkgs.python311;
  uv.enable = true;
  venv.enable = true;
  venv.quiet = true;
};
```

Only emit configured fields.

- [ ] **Step 2: Run rendering tests**

Run: `cargo test --test generator_tests`
Expected: PASS

- [ ] **Step 3: Run planning tests**

Run: `cargo test --test generate_rust_file_tests`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add templates/devenv.nix.tera tests/generator_tests.rs tests/generate_rust_file_tests.rs
git commit -m "feat: render python devenv config"
```

### Task 5: Full Verification

**Files:**
- Modify: none
- Test: `tests/generator_tests.rs`
- Test: `tests/generate_rust_file_tests.rs`

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: PASS with all tests green.

- [ ] **Step 2: Review generated behavior against the approved scope**

Check that implementation includes:

- `version`
- `package`
- `uv.enable`
- `venv.enable`
- `venv.quiet`

Check that it excludes:

- `venv.requirements`
- explicit `null` output

- [ ] **Step 3: Commit**

```bash
git add .
git commit -m "feat: add minimal python init support"
```
