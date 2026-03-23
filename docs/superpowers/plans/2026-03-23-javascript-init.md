# JavaScript Init Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the legacy Nodejs naming with JavaScript and add a minimal interactive JavaScript init flow that covers Node package selection plus common package-manager toggles.

**Architecture:** Extend the existing enum-based language model and prompt-driven CLI flow, following the same structure used by Rust, Python, Go, and Java. Reuse the current Tera rendering pipeline so JavaScript support slots into the existing `ProjectContext -> templates -> files` path with no extra config layer.

**Tech Stack:** Rust, clap, dialoguer, serde, tera, cargo test

---

### Task 1: Add Failing JavaScript Rendering Tests

**Files:**
- Modify: `tests/generator_tests.rs`
- Test: `tests/generator_tests.rs`

- [ ] **Step 1: Write the failing tests**

Add tests for:

```rust
#[test]
fn test_render_javascript_base() { /* expect languages.javascript.enable = true */ }

#[test]
fn test_render_javascript_with_package() { /* expect package = pkgs.nodejs_22 */ }

#[test]
fn test_render_javascript_with_npm() { /* expect npm.enable = true */ }

#[test]
fn test_render_javascript_with_pnpm() { /* expect pnpm.enable = true */ }

#[test]
fn test_render_javascript_with_yarn() { /* expect yarn.enable = true */ }

#[test]
fn test_render_javascript_with_corepack() { /* expect corepack.enable = true */ }

#[test]
fn test_render_javascript_with_bun() { /* expect bun.enable = true */ }

#[test]
fn test_render_devenv_yaml_for_javascript() { /* expect only base nixpkgs input */ }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test generator_tests`
Expected: FAIL because JavaScript schema and templates do not yet exist, and Nodejs naming is still present.

- [ ] **Step 3: Commit**

```bash
git add tests/generator_tests.rs
git commit -m "test: add failing javascript rendering tests"
```

### Task 2: Add Failing JavaScript File Planning Test

**Files:**
- Modify: `tests/generate_rust_file_tests.rs`
- Test: `tests/generate_rust_file_tests.rs`

- [ ] **Step 1: Write the failing test**

Add a JavaScript planning test that asserts:

- `plan_files` returns 3 files
- filenames include `devenv.nix`, `devenv.yaml`, `.envrc`
- `devenv.nix` contains `languages.javascript`

- [ ] **Step 2: Run tests to verify it fails**

Run: `cargo test --test generate_rust_file_tests`
Expected: FAIL because JavaScript output is not yet rendered.

- [ ] **Step 3: Commit**

```bash
git add tests/generate_rust_file_tests.rs
git commit -m "test: add failing javascript planning test"
```

### Task 3: Rename Nodejs and Extend the JavaScript Schema

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/schema/context.rs`
- Modify: `src/prompt.rs`
- Modify: `src/main.rs`
- Test: `tests/generator_tests.rs`

- [ ] **Step 1: Rename Nodejs to JavaScript**

Rename:

- `LanguageChoice::Nodejs` -> `LanguageChoice::JavaScript`
- `Language::Nodejs` -> `Language::JavaScript`
- menu labels from `Nodejs` to `JavaScript`

- [ ] **Step 2: Implement the minimal schema**

Change `Language::JavaScript` to:

```rust
JavaScript {
    #[serde(skip_serializing_if = "Option::is_none")]
    package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    npm_enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pnpm_enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    yarn_enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    corepack_enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bun_enable: Option<bool>,
}
```

- [ ] **Step 3: Add the JavaScript prompt function**

Implement `prompt_javascript_config()` with:

- default confirmation path
- optional `package`
- `enable npm?`
- `enable pnpm?`
- `enable yarn?`
- `enable corepack?`
- `enable bun?`

- [ ] **Step 4: Wire the CLI to the new prompt**

Import `prompt_javascript_config()` in `src/main.rs` and use it in the JavaScript branch.

- [ ] **Step 5: Run targeted tests**

Run: `cargo test --test generator_tests`
Expected: still FAIL because templates do not render JavaScript yet, but compile-time model errors are resolved.

- [ ] **Step 6: Commit**

```bash
git add src/cli.rs src/schema/context.rs src/prompt.rs src/main.rs
git commit -m "feat: add javascript init schema and prompts"
```

### Task 4: Render JavaScript Config

**Files:**
- Modify: `templates/devenv.nix.tera`
- Test: `tests/generator_tests.rs`
- Test: `tests/generate_rust_file_tests.rs`

- [ ] **Step 1: Implement JavaScript rendering in devenv.nix**

Add a JavaScript branch that emits:

```nix
languages.javascript = {
  enable = true;
  package = pkgs.nodejs_22;
  npm.enable = true;
  pnpm.enable = true;
  yarn.enable = true;
  corepack.enable = true;
  bun.enable = true;
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
git commit -m "feat: render javascript devenv config"
```

### Task 5: Full Verification

**Files:**
- Modify: none
- Test: `tests/generator_tests.rs`
- Test: `tests/generate_rust_file_tests.rs`

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: PASS with all tests green.

- [ ] **Step 2: Review generated behavior against scope**

Check that implementation includes:

- `package`
- `npm.enable`
- `pnpm.enable`
- `yarn.enable`
- `corepack.enable`
- `bun.enable`
- `Nodejs` renamed to `JavaScript`

Check that it excludes:

- `bun.package`
- `*.install.enable`
- `directory`
- `lsp.*`
- extra `devenv.yaml` inputs for JavaScript

- [ ] **Step 3: Commit**

```bash
git add .
git commit -m "feat: add minimal javascript init support"
```
