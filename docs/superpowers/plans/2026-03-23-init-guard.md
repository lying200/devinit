# Init Guard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent `devinit` from initializing directories that already contain `devenv`, `direnv`, or Nix environment markers, and run `git init` only when the initialization target is completely empty.

**Architecture:** Add a focused `init_guard` module that owns directory inspection, empty-directory checks, and Git initialization helpers. Wire `main.rs` to create missing target directories first, run the guard before prompts, initialize Git only for truly empty targets, then continue with existing file generation and ignore handling.

**Tech Stack:** Rust, clap, dialoguer, std::fs, std::process::Command, cargo test

---

### Task 1: Add Failing Init Guard Tests

**Files:**
- Modify: `tests/init_guard_tests.rs`
- Modify: `src/lib.rs`
- Test: `tests/init_guard_tests.rs`

- [ ] **Step 1: Write the failing tests**

Add or update tests for:

```rust
#[test]
fn detect_existing_environment_returns_none_for_missing_directory() { /* expect None */ }

#[test]
fn target_dir_was_empty_returns_true_for_missing_directory() { /* expect true */ }

#[test]
fn target_dir_was_empty_returns_true_for_empty_directory() { /* expect true */ }

#[test]
fn target_dir_was_empty_returns_false_for_non_empty_directory() { /* expect false */ }

#[test]
fn devinit_creates_missing_target_dir_before_prompting() { /* expect directory created */ }

#[test]
fn devinit_skips_initialization_when_existing_environment_is_detected() { /* expect success skip */ }
```

Keep the tests focused on the new behavior. Remove or replace the old assertions that depend on `default_tools_for_target`.

- [ ] **Step 2: Run the new test target to verify it fails**

Run: `cargo test --test init_guard_tests`
Expected: FAIL because the empty-directory helpers and Git-init behavior do not exist yet.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs tests/init_guard_tests.rs
git commit -m "test: update init guard tests for git init flow"
```

### Task 2: Add Failing Git Init Integration Tests

**Files:**
- Modify: `tests/init_guard_tests.rs`
- Test: `tests/init_guard_tests.rs`

- [ ] **Step 1: Write the failing binary integration tests**

Add tests that execute the compiled binary:

```rust
#[test]
fn devinit_initializes_git_for_missing_directory() {
    // missing dir
    // run CARGO_BIN_EXE_devinit --lang rust <dir>
    // expect .git created before prompt or by init path
}

#[test]
fn devinit_initializes_git_for_existing_empty_directory() {
    // empty dir
    // run binary
    // expect .git created
}

#[test]
fn devinit_does_not_initialize_git_for_non_empty_directory() {
    // create README.md
    // run binary
    // expect no .git directory
}
```

The tests may assert on side effects before prompt completion rather than requiring a full successful interactive run.

- [ ] **Step 2: Run the test target to verify it fails for the right reason**

Run: `cargo test --test init_guard_tests`
Expected: FAIL because the current binary does not yet run `git init` only for empty directories.

- [ ] **Step 3: Commit**

```bash
git add tests/init_guard_tests.rs
git commit -m "test: add failing git init behavior tests"
```

### Task 3: Implement Empty-Directory and Git Init Helpers

**Files:**
- Modify: `src/init_guard.rs`
- Modify: `src/lib.rs`
- Test: `tests/init_guard_tests.rs`

- [ ] **Step 1: Implement the public API**

Create or update:

```rust
pub fn detect_existing_environment(target_dir: &Path) -> io::Result<Option<String>>;
pub fn target_dir_was_empty(target_dir: &Path, existed_before: bool) -> io::Result<bool>;
pub fn initialize_git_repository(target_dir: &Path) -> io::Result<()>;
```

Behavior:

- missing directory => no existing environment
- missing directory counts as empty when `existed_before` is `false`
- existing directory counts as empty only when it has zero entries
- `initialize_git_repository` shells out to `git init <target_dir>`

- [ ] **Step 2: Remove obsolete helper usage**

Drop any helper that only existed for `ProjectContext.tools` injection if it is no longer needed.

- [ ] **Step 3: Run the init guard tests**

Run: `cargo test --test init_guard_tests`
Expected: helper-level tests pass, but binary integration tests may still fail until `main.rs` is fully wired.

- [ ] **Step 4: Commit**

```bash
git add src/init_guard.rs src/lib.rs tests/init_guard_tests.rs
git commit -m "feat: add empty-directory git init helpers"
```

### Task 4: Wire the Main Init Flow

**Files:**
- Modify: `src/main.rs`
- Test: `tests/init_guard_tests.rs`

- [ ] **Step 1: Create missing target directories first**

At the top of `main.rs`:

```rust
let target_dir = cli.path;
let existed_before = target_dir.exists();
std::fs::create_dir_all(&target_dir)?;
```

Preserve the existing stderr + non-zero exit style for failure.

- [ ] **Step 2: Add the preflight guard**

Keep the existing environment detection before any prompt:

```rust
if let Some(found) = detect_existing_environment(&target_dir)? {
    println!("existing direnv/devenv/nix environment detected ({found}), skipping devinit initialization");
    return;
}
```

- [ ] **Step 3: Run `git init` only for empty targets**

Use the new helper:

```rust
if target_dir_was_empty(&target_dir, existed_before)? {
    initialize_git_repository(&target_dir)?;
}
```

- [ ] **Step 4: Remove old default git injection**

Set:

```rust
let ctx = ProjectContext {
    language,
    services: vec![],
    tools: vec![],
};
```

Do not inject `git` through `ProjectContext.tools`.

- [ ] **Step 5: Run the init guard test target**

Run: `cargo test --test init_guard_tests`
Expected: PASS, including the git-init integration tests.

- [ ] **Step 6: Commit**

```bash
git add src/main.rs tests/init_guard_tests.rs
git commit -m "feat: init git for empty targets"
```

### Task 5: Update Rendering Coverage

**Files:**
- Modify: `tests/generator_tests.rs`
- Test: `tests/generator_tests.rs`

- [ ] **Step 1: Replace obsolete default-git rendering tests**

Update tests so they no longer assume `git` is injected by default. Keep only coverage that proves the generator renders tools when explicitly present.

- [ ] **Step 2: Run the rendering tests**

Run: `cargo test --test generator_tests`
Expected: FAIL if the old injected-git expectation is still present.

- [ ] **Step 3: Make the minimal test updates**

Do not change generator implementation unless the renderer itself is broken.

- [ ] **Step 4: Re-run the rendering tests**

Run: `cargo test --test generator_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/generator_tests.rs
git commit -m "test: update generator expectations for git init flow"
```

### Task 6: Full Verification

**Files:**
- Modify: none
- Test: `tests/init_guard_tests.rs`
- Test: `tests/generator_tests.rs`
- Test: `tests/generate_rust_file_tests.rs`
- Test: `tests/git_ignore_tests.rs`

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: PASS with init guard, generator, file planning, and ignore-handling tests green.

- [ ] **Step 2: Run static verification**

Run: `cargo fmt --check`
Expected: PASS

Run: `cargo clippy --tests -- -D warnings`
Expected: PASS

- [ ] **Step 3: Review the behavior against the spec**

Check that:

- missing target directories are created first
- existing environment markers still cause an early skip
- only truly empty targets trigger `git init`
- non-empty targets do not trigger `git init`
- generated config no longer relies on default `git` package injection

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "feat: initialize git for empty devinit targets"
```
