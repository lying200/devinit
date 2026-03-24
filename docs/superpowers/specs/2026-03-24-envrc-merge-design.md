# Envrc Merge Design

## Goal

Allow `devinit` to work with an existing project-level `.envrc` by appending a `devinit`-managed block when needed, while keeping repeated runs idempotent.

## Scope

This design supports only:

- creating `.envrc` when it does not exist
- appending a `devinit` block to an existing `.envrc` when that block is missing
- skipping `.envrc` changes when the managed block already exists
- keeping the existing skip behavior for other `devenv`, `direnv`, and Nix environment markers

This design explicitly does not support:

- parsing arbitrary shell code in `.envrc`
- reordering existing `.envrc` content
- merging multiple different `devinit` blocks
- migrating existing non-`devinit` `devenv` snippets into the managed block

## Why This Scope

The current init flow treats `.envrc` as an unconditional environment marker and exits before initialization. Even if that guard were removed, the file-writing layer would still overwrite `.envrc` with the generated template.

That behavior is too blunt for repositories that already use `.envrc` for unrelated shell setup. The desired behavior is narrower:

- keep protecting directories that already have a full `devenv`, `direnv`, or Nix setup
- allow a standalone `.envrc` file to be extended rather than treated as a hard conflict
- make repeated `devinit` runs safe by avoiding duplicate managed content

## Current Project Structure

The current behavior is split across:

- `src/init_guard.rs`, which identifies top-level environment markers
- `src/main.rs`, which exits early when the guard finds a marker
- `src/generator.rs`, which plans generated files and writes them directly with `std::fs::write`
- `templates/.envrc.tera`, which defines the `devenv` shell snippet

The change should stay in the init and file-writing layers. Template rendering can remain simple.

## Approaches Considered

### Recommended: Dedicated `.envrc` merge path

Handle `.envrc` separately from the generic file writer:

- if `.envrc` is missing, create it from the existing template
- if `.envrc` exists and already contains a managed `devinit` block, leave it untouched
- if `.envrc` exists and does not contain the managed block, append a managed block at the end

Why this is recommended:

- matches the desired user-facing behavior
- keeps merge semantics narrow and predictable
- avoids parsing arbitrary shell syntax

### Alternative: Continue to skip when `.envrc` exists

Why this is not recommended:

- blocks a common and reasonable repository state
- forces manual merging for a case that can be handled safely

### Alternative: Fully parse and rewrite `.envrc`

Why this is not recommended:

- shell parsing is fragile and unnecessary for the stated goal
- increases the chance of breaking user-managed logic

## Managed Block Format

The appended section should use explicit markers so the tool can detect it on later runs.

Recommended block shape:

```bash
# devinit: start
eval "$(devenv direnvrc)"

# You can pass flags to the devenv command
# For example: use devenv --impure --option services.postgres.enable:bool true
use devenv
# devinit: end
```

Rules:

- the managed block is detected by the start and end markers
- block contents come from the existing `.envrc` template body
- the tool does not modify content outside the managed block

## Existing Environment Detection

Initialization should still stop when the target directory contains:

- `devenv.nix`
- `devenv.yaml`
- `devenv.lock`
- `.devenv*`
- `.direnv`
- `shell.nix`
- `default.nix`
- `flake.nix`
- `flake.lock`

`.envrc` should no longer be treated as a standalone hard-stop marker.

This keeps the previous conservative behavior for full environment setups while permitting `.envrc` extension.

## File Writing Behavior

Recommended responsibility split:

- keep `plan_files` responsible for rendered target content
- update `write_files` so `.envrc` is handled by a dedicated helper
- keep all other files on the existing overwrite path

The `.envrc` helper should:

1. read the current file if it exists
2. return early if the managed block markers are already present
3. append the managed block with clean newline separation when the block is missing
4. create the file directly when it does not exist

## Error Handling

Expected cases:

- `.envrc` missing: create it normally
- `.envrc` readable and missing block: append managed block
- `.envrc` readable and already managed: leave unchanged
- `.envrc` unreadable or unwritable: return the I/O error and preserve current non-zero CLI behavior

## Testing Strategy

Follow TDD and cover the behavior in two layers.

Add focused generator/file-writing tests for:

- creating `.envrc` from scratch
- appending a managed block to an existing `.envrc`
- leaving an existing managed `.envrc` unchanged

Add integration coverage for:

- `.envrc` alone no longer causes the CLI to skip initialization
- other environment markers such as `flake.nix` still cause an early skip

## Risks

- marker strings become part of the public file format, so they should stay stable once released
- appending at file end may not match every user's preferred ordering, but it is safer than rewriting surrounding shell code
- an existing `.envrc` that already contains equivalent `devenv` commands without markers will still get a managed block appended

## Non-Goals

- deduplicating arbitrary user-written `devenv` commands
- editing or removing an existing managed block
- supporting multiple independently managed blocks in one `.envrc`
