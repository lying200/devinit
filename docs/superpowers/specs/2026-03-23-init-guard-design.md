# Init Guard Design

## Goal

Add a pre-initialization guard that prevents `devinit` from running in directories that already contain `devenv`, `direnv`, or Nix environment markers, and automatically run `git init` only when the initialization target is completely empty.

## Scope

This design supports only:

- detecting an existing local `devenv` environment before any initialization prompt runs
- detecting an existing `direnv + nix` environment before any initialization prompt runs
- exiting early when such an environment is present
- creating `target_dir` before guard checks when it does not already exist
- running `git init` only when the target directory was completely empty before initialization

This design explicitly does not support:

- searching parent directories for Git repositories
- inspecting shell command contents inside `.envrc`
- auto-migrating existing Nix or `direnv` setups into `devenv`
- partially initializing a directory that already contains environment markers
- adding `git` to generated `devenv.nix` as a fallback behavior

## Why This Scope

The CLI currently initializes `devenv` in place without checking whether the directory is already managed by `devenv`, `direnv`, or a Nix-based shell setup.

That creates two unwanted outcomes:

- `devinit` can overwrite or conflict with an existing environment setup
- users working in brand new directories still need to initialize Git by hand, even though an empty project directory is the clearest case for automatic repository setup

The desired behavior is conservative:

- if environment markers already exist, skip initialization entirely
- if the target directory is completely empty, initialize Git automatically
- if the target directory is not empty, do not initialize Git unless it already exists

## Current Project Structure

The current init flow is centered in `src/main.rs`:

- `src/main.rs` selects the language, builds `ProjectContext`, writes generated files, and runs ignore handling
- `src/generator.rs` turns `ProjectContext` into `devenv.nix`, `devenv.yaml`, and `.envrc`
- `src/git_ignore.rs` checks whether the current target directory has `.git`

The new behavior belongs before language prompts and before file generation. It should not be embedded in the template rendering layer.

## Approaches Considered

### Recommended: Preflight guard in the main init flow

Add a small dedicated module that inspects the target directory before any prompt runs and lets the main flow decide whether to run `git init`.

Why this is recommended:

- avoids prompting the user for configuration that will never be applied
- keeps early-exit logic separate from file generation
- allows the same early inspection to decide whether `git init` should run

### Alternative: Detect only file overwrite conflicts

Exit only when generated target files such as `devenv.nix` or `.envrc` already exist.

Why this is not recommended:

- misses the user's explicit requirement that `direnv + nix` setups should also block initialization
- treats unrelated marker files such as `flake.nix` as acceptable when they are not

### Alternative: Keep injecting `git` into generated config

Continue to add `git` through `ProjectContext.tools` when `.git` is missing.

Why this is not recommended:

- the desired behavior has changed from "make git available" to "initialize Git only for truly empty directories"
- it leaves two partially overlapping Git behaviors in the codebase

## Existing Environment Detection

Initialization must stop when the target directory already contains any of these markers:

- `devenv.nix`
- `devenv.yaml`
- `devenv.lock`
- `.devenv*`
- `.direnv`
- `.envrc`
- `shell.nix`
- `default.nix`
- `flake.nix`
- `flake.lock`

Detection rules:

- check only the target directory itself
- do not search parent directories
- `.devenv*` means any file or directory in the target directory whose name starts with `.devenv`
- `.direnv` means the top-level `.direnv` directory or file in the target directory

If any marker is found:

- print a message such as `existing direnv/devenv/nix environment detected (<match>), skipping devinit initialization`
- exit immediately
- do not prompt for language selection
- do not write any files
- do not run ignore handling

Recommended exit code: `0`, because the skip is intentional rather than a failed execution.

## Empty Directory Rule

The Git initialization check is intentionally local.

Rules:

- capture whether `target_dir` existed before the command started
- if it did not exist, create it first and treat it as an empty directory
- if it already existed, inspect only that directory's direct entries
- a directory counts as completely empty only when it has zero entries
- any file or subdirectory means it is not empty

The design explicitly does not look at parent directories when deciding whether to run `git init`.

This matches the desired semantics: only the current initialization target matters.

## Git Initialization Behavior

If initialization proceeds and the target directory was completely empty before initialization, `devinit` should run:

```bash
git init <target_dir>
```

If the target directory was not completely empty, `devinit` must not run `git init`.

If `git` is unavailable on the system or `git init` fails, the command should print a clear error and exit with a non-zero status.

This replaces the previous idea of injecting `git` into generated `devenv.nix`. Git setup is now an initialization-side effect, not a rendered package default.

## Components

Recommended units:

- a new module such as `src/init_guard.rs`
- `detect_existing_environment(target_dir: &Path) -> io::Result<Option<String>>`
- `target_dir_was_empty(target_dir: &Path) -> io::Result<bool>` or equivalent helper
- `initialize_git_repository(target_dir: &Path) -> io::Result<()>` or equivalent helper

Responsibility split:

- `init_guard` owns directory inspection
- `main.rs` owns control flow, exit behavior, and the decision to run `git init`
- `generator.rs` continues to consume `ProjectContext` and render files

## Main Flow Integration

The updated `main.rs` flow becomes:

1. resolve `target_dir`
2. record whether the target directory existed before the command started
3. create `target_dir` if it does not yet exist
4. check for existing `devenv` / `direnv` / Nix environment markers
5. if found, print the skip message and exit
6. determine whether the target directory was completely empty before initialization
7. if it was empty, run `git init`
8. collect language selection and prompt inputs
9. generate and write files
10. run the existing ignore handling flow

This ordering is important because users should not answer prompts when initialization will be skipped anyway.

## Error Handling

Expected cases:

- `target_dir` does not exist: create it and continue
- directory inspection succeeds and finds no markers: continue normally
- directory inspection succeeds and finds a marker: print skip message and exit with code `0`
- directory inspection fails because the directory cannot be read: print an error and exit with a non-zero status
- target directory is empty and `git init` succeeds: continue normally
- target directory is empty and `git` is unavailable or `git init` fails: print an error and exit with a non-zero status
- file generation or ignore handling fails later: preserve the existing non-zero exit behavior

## Testing Strategy

Follow TDD and cover the behavior in two layers.

Add focused tests for the new guard module:

- detects `devenv.nix`
- detects `.envrc`
- detects `flake.nix`
- detects `.devenv`-prefixed entries
- detects `.direnv`
- reports no marker for a clean directory
- reports no marker for a missing directory
- distinguishes empty from non-empty target directories

Add integration-style coverage for the init flow:

- a missing `target_dir` is created before prompting begins
- a missing or empty directory triggers `git init`
- a non-empty directory does not trigger `git init`
- when the directory contains an existing marker, initialization exits before writing files
- when the directory contains an existing marker, ignore handling does not run
- when `git` is unavailable for an empty directory, the command exits non-zero with a clear error

## Risks

- `.envrc` may exist for non-Nix workflows, but the selected product rule is still to skip initialization
- some users may expect parent Git repositories to count, but this design intentionally does not
- automatic `git init` adds a side effect before prompting, so tests must verify it happens only for truly empty directories
- environments without `git` installed now have an explicit hard dependency in the empty-directory path

## Non-Goals

- parsing `.envrc` to determine whether it references Nix or `devenv`
- supporting nested project directories inside a parent repository
- inferring project intent from arbitrary shell files
- changing the existing language prompt structure
