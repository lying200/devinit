# Devenv Ignore Handling Design

## Goal

Add a post-generation ignore-handling step to the CLI so users can choose whether `devenv`-related files should be ignored, while preserving a local-only path for teams that do not want to commit the `devenv` mechanism to the shared repository.

## Scope

This design supports only:

- a post-generation three-way ignore choice
- writing shared ignore rules to `.gitignore`
- writing local-only ignore rules to `.git/info/exclude`
- skipping all ignore handling when the target directory is not inside a Git repository
- warning when ignored files are already tracked by Git

This design explicitly does not support:

- a standalone `ignore` subcommand
- removing tracked files from the Git index
- editing parent-directory `.gitignore` files
- custom ignore pattern editing by the user

## Why This Scope

The project currently generates `devenv.nix`, `devenv.yaml`, and `.envrc`, but it does not help users manage the common `devenv` and `direnv` ignore patterns that usually follow initialization.

The design needs two distinct outcomes:

- a repository-level ignore option for local state such as `.devenv*` and `.direnv`
- a local-only option for users who want to keep the entire `devenv` mechanism out of the shared repository because remote collaborators may not use Nix

The local-only mode must not mutate shared ignore files. That is the central product constraint.

## Current Project Structure

The existing initialization flow already has a natural insertion point:

- `src/main.rs` handles the interactive CLI flow and writes generated files
- `src/prompt.rs` contains prompt logic using `dialoguer`
- `src/generator.rs` plans and writes generated `devenv` files

This feature should be added after file generation succeeds. It should not be merged into the template-rendering logic because ignore handling is a separate concern.

## Approaches Considered

### Recommended: Post-generation ignore selection

After `write_files(...)` succeeds, prompt the user to choose one of three ignore modes.

Why this is recommended:

- matches the real user flow directly after `devenv` files are created
- keeps the behavior discoverable without adding a new command
- makes the local-only mode explicit at the moment it matters

### Alternative: Standalone ignore command

Add a separate command such as `devinit ignore`.

Why this is not recommended:

- users must remember an extra command
- it separates ignore handling from the initialization path that produces the relevant files
- it adds CLI surface area before the current core flow is settled

### Alternative: Nested confirm prompts

Ask a chain of yes/no questions instead of using a single select prompt.

Why this is not recommended:

- the interaction becomes harder to scan
- the user already wants a clear three-way choice
- the local-only mode no longer reads as a distinct product decision

## Git Repository Precondition

Ignore handling only runs when the target directory is inside a Git repository.

Behavior:

- resolve the target directory used for generated file output
- detect whether that directory belongs to a Git repository
- if no repository is found, skip the entire ignore flow
- print a clear message: `git not initialized, skipping ignore handling`

This rule applies to both `.gitignore` and `.git/info/exclude` modes. If Git is not initialized, no ignore files are created or modified.

## User Interaction

After file generation succeeds and the Git repository precondition passes, prompt:

`How to handle git ignore for devenv files?`

Options:

- `Do nothing`
- `Add to .gitignore`
- `Add to local git exclude (.git/info/exclude, ignore devenv mechanism locally)`

Meaning of each option:

- `Do nothing`: no ignore files are changed
- `Add to .gitignore`: add the shared ignore set for local `devenv` state
- `Add to local git exclude ...`: add a larger local-only ignore set that ignores the `devenv` mechanism itself

The local-only mode does not ask a follow-up question. Choosing it already means the user wants to ignore the `devenv` mechanism locally.

## Ignore Rule Sets

### Shared `.gitignore` rule set

This mode writes only the local-state rules:

```gitignore
# devinit: devenv ignores
.devenv*
devenv.local.nix
devenv.local.yaml
.direnv
```

### Local `.git/info/exclude` rule set

This mode writes the shared local-state rules plus the core `devenv` mechanism files:

```gitignore
# devinit: local devenv mechanism ignores
.devenv*
devenv.local.nix
devenv.local.yaml
.direnv
devenv.nix
devenv.yaml
devenv.lock
.envrc
```

Rationale:

- `.devenv*`, `devenv.local.*`, and `.direnv` are disposable local state
- `devenv.nix`, `devenv.yaml`, `devenv.lock`, and `.envrc` represent the core local `devenv` mechanism that some users do not want to publish

## File Resolution

When ignore handling is enabled:

- `.gitignore` mode writes to `target_dir/.gitignore`
- `local git exclude` mode finds the repository root for `target_dir` and writes to `<repo_root>/.git/info/exclude`

This split is intentional:

- `.gitignore` should sit beside the generated project files
- `.git/info/exclude` is repository-local by Git design and must be written at the repository root

## Write Semantics

The ignore writer should be idempotent.

Rules:

- read the destination file if it exists
- compare line-by-line using exact matches
- append only missing rules
- do not duplicate existing rules
- if all target rules already exist, treat the operation as a no-op
- when appending, insert a single separating blank line if needed

A small helper module such as `src/git_ignore.rs` should own this logic. The existing generator module should remain focused on generated file planning and output.

## Tracked File Detection

Ignore rules only affect untracked files. Already tracked files remain tracked by Git even if matching patterns are added later.

Because of that, both writing modes should also detect whether any relevant files are already tracked in the target directory.

Relevant tracked files to check:

- `.devenv*`
- `devenv.local.nix`
- `devenv.local.yaml`
- `.direnv`
- `devenv.nix`
- `devenv.yaml`
- `devenv.lock`
- `.envrc`

If tracked files are found:

- still write the selected ignore rules
- print a warning that tracked files remain tracked by Git
- list the matching tracked paths

This feature must not automatically run `git rm --cached` because that would be an index mutation the user did not explicitly request.

## Components

Recommended units:

- `prompt_ignore_mode()` in `src/prompt.rs` for the three-way selection
- a new `IgnoreMode` enum to represent the chosen behavior
- a new `src/git_ignore.rs` module for repository detection, rule selection, file writing, and tracked-file inspection
- a small result type that lets `main.rs` print accurate status messages

## Main Flow Integration

The updated `main.rs` flow becomes:

1. collect language selection and config
2. generate the base `devenv` files
3. check whether the target directory is inside a Git repository
4. if not, print `git not initialized, skipping ignore handling`
5. otherwise prompt for ignore mode
6. apply the chosen ignore mode
7. print success and any tracked-file warning

This keeps the failure boundary narrow: `devenv` file generation remains the primary success path, and ignore handling is an optional follow-up step.

## Error Handling

Expected cases:

- target directory is not inside Git: skip ignore flow with a message
- `.gitignore` cannot be read or written: return an error and fail the command
- `.git/info/exclude` cannot be read or written: return an error and fail the command
- Git tracked-file inspection fails unexpectedly: return an error and fail the command

The only non-error skip path is the missing Git repository case, because that is now an explicit product rule.

## Testing Strategy

Follow TDD and add focused coverage for both the pure write logic and the CLI integration path.

Add unit or module tests for:

- shared rule writing to a new `.gitignore`
- shared rule writing without duplicating existing entries
- local exclude writing to `.git/info/exclude`
- no-op behavior when all target rules already exist
- repository detection from a nested target directory
- skip behavior when the target directory is not inside a Git repository

Add integration-style coverage for:

- local exclude mode includes `devenv.nix`, `devenv.yaml`, `devenv.lock`, and `.envrc`
- `.gitignore` mode includes only `.devenv*`, `devenv.local.*`, and `.direnv`
- tracked-file detection reports already tracked files without attempting index cleanup

## Risks

- tracked-file detection may require shelling out to Git, which introduces command-execution edge cases
- users may expect `.gitignore` mode to modify a repository-root ignore file instead of `target_dir/.gitignore`
- local exclude mode can give the impression that tracked files will disappear from status, so the warning must be explicit

## Non-Goals

- automatically untracking files
- teaching Git ignore semantics inside the prompt text
- adding ignore handling for unrelated tools or services
- changing which base `devenv` files the project generates
