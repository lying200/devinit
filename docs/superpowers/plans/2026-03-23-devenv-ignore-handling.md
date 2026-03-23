# Devenv Ignore Handling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a post-generation ignore-handling flow that offers three choices, skips entirely when Git is not initialized, and supports local-only `devenv` mechanism ignores via `.git/info/exclude`.

**Architecture:** Keep generated-file planning and writing in `src/generator.rs`, then add a separate Git-ignore module to handle repository detection, ignore rule selection, idempotent file updates, and tracked-file warnings. Extend the existing `dialoguer` prompt flow in `src/main.rs` so ignore handling runs only after successful file generation.

**Tech Stack:** Rust, clap, dialoguer, std::fs, std::process::Command, cargo test

---

### Task 1: Add Failing Git Ignore Module Tests

**Files:**
- Modify: `Cargo.toml`
- Create: `tests/git_ignore_tests.rs`
- Test: `tests/git_ignore_tests.rs`

- [ ] **Step 1: Write the failing tests**

Add integration-style tests covering:

```rust
#[test]
fn apply_gitignore_rules_creates_gitignore_in_target_dir() { /* expect shared rules */ }

#[test]
fn apply_gitignore_rules_does_not_duplicate_existing_entries() { /* expect deduped content */ }

#[test]
fn apply_local_exclude_rules_writes_repo_root_exclude() { /* expect local-only rules */ }

#[test]
fn skip_ignore_handling_when_git_is_not_initialized() { /* expect skip result */ }

#[test]
fn tracked_files_are_reported_without_removing_them() { /* expect tracked list */ }
```

Use temporary directories plus `git init` where needed. Shell out to Git from the tests only to create realistic repositories and tracked files.

- [ ] **Step 2: Run the new test target to verify it fails**

Run: `cargo test --test git_ignore_tests`
Expected: FAIL because the `git_ignore` module, result types, and helper functions do not exist yet.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml tests/git_ignore_tests.rs
git commit -m "test: add failing git ignore handling tests"
```

### Task 2: Add Failing Prompt Selection Test

**Files:**
- Modify: `src/prompt.rs`
- Modify: `tests/git_ignore_tests.rs`
- Test: `tests/git_ignore_tests.rs`

- [ ] **Step 1: Add a narrow prompt-facing test seam**

Add a testable helper in `src/prompt.rs` that maps a selected index to the ignore mode enum:

```rust
fn ignore_mode_from_selection(selection: usize) -> IgnoreMode
```

Do not implement behavior yet beyond what is needed for the failing test.

- [ ] **Step 2: Add the failing test**

Add a test that asserts:

- `0 -> IgnoreMode::None`
- `1 -> IgnoreMode::GitIgnore`
- `2 -> IgnoreMode::LocalExclude`

- [ ] **Step 3: Run the new test target to verify it fails**

Run: `cargo test --test git_ignore_tests`
Expected: FAIL because `IgnoreMode` and the mapping helper are not implemented.

- [ ] **Step 4: Commit**

```bash
git add src/prompt.rs tests/git_ignore_tests.rs
git commit -m "test: add failing ignore mode selection test"
```

### Task 3: Implement the Git Ignore Module

**Files:**
- Create: `src/git_ignore.rs`
- Modify: `src/lib.rs`
- Test: `tests/git_ignore_tests.rs`

- [ ] **Step 1: Implement the minimal types**

Create:

```rust
pub enum IgnoreMode {
    None,
    GitIgnore,
    LocalExclude,
}

pub struct IgnoreOutcome {
    pub skipped_for_missing_git: bool,
    pub destination: Option<PathBuf>,
    pub wrote_rules: bool,
    pub tracked_files: Vec<String>,
}
```

Keep the API small and aligned with the spec.

- [ ] **Step 2: Implement repository detection**

Add a helper that walks from `target_dir` upward and detects the Git repository root by locating `.git`.

- [ ] **Step 3: Implement shared and local rule selection**

Return the exact rule sets from the spec:

- shared `.gitignore` rules for `.devenv*`, `devenv.local.nix`, `devenv.local.yaml`, `.direnv`
- local exclude rules for the shared set plus `devenv.nix`, `devenv.yaml`, `devenv.lock`, `.envrc`

- [ ] **Step 4: Implement idempotent file writing**

Write only missing lines, preserving existing file content and avoiding duplicate rules.

- [ ] **Step 5: Implement tracked-file inspection**

Use a minimal `git ls-files` command scoped to the relevant patterns. Return tracked paths without altering Git state.

- [ ] **Step 6: Run the git ignore tests**

Run: `cargo test --test git_ignore_tests`
Expected: PASS for all module-level behavior except any prompt or main-flow integration still not wired.

- [ ] **Step 7: Commit**

```bash
git add src/git_ignore.rs src/lib.rs tests/git_ignore_tests.rs
git commit -m "feat: add git ignore handling module"
```

### Task 4: Wire the Prompt and Main Flow

**Files:**
- Modify: `src/prompt.rs`
- Modify: `src/main.rs`
- Test: `tests/git_ignore_tests.rs`

- [ ] **Step 1: Implement the prompt helper and interactive prompt**

Add:

```rust
pub fn prompt_ignore_mode() -> IgnoreMode
```

Use `dialoguer::Select` with:

- `Do nothing`
- `Add to .gitignore`
- `Add to local git exclude (.git/info/exclude, ignore devenv mechanism locally)`

Back it with the tested selection-mapping helper.

- [ ] **Step 2: Integrate ignore handling into `main.rs`**

Update the flow:

1. generate files
2. check whether the target directory is inside a Git repo
3. if not, print `git not initialized, skipping ignore handling`
4. otherwise prompt for ignore mode
5. apply the selected mode
6. print any tracked-file warning

- [ ] **Step 3: Run the git ignore tests again**

Run: `cargo test --test git_ignore_tests`
Expected: PASS

- [ ] **Step 4: Run the existing test suites**

Run: `cargo test --test generator_tests`
Expected: PASS

Run: `cargo test --test generate_rust_file_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/prompt.rs src/main.rs
git commit -m "feat: prompt for devenv ignore handling"
```

### Task 5: Full Verification

**Files:**
- Modify: none
- Test: `tests/git_ignore_tests.rs`
- Test: `tests/generator_tests.rs`
- Test: `tests/generate_rust_file_tests.rs`

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: PASS with the new ignore-handling tests and all existing tests green.

- [ ] **Step 2: Review implementation against the spec**

Check that:

- Git-missing directories skip all ignore handling
- `.gitignore` mode writes only the shared local-state rules
- local exclude mode writes the expanded local-only rules
- tracked files are only reported, never untracked automatically

- [ ] **Step 3: Commit**

```bash
git add .
git commit -m "feat: add devenv ignore handling"
```
