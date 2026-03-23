# Go Init Design

## Goal

Add a first minimal Go initialization flow to the CLI, aligned with the existing Rust and Python flows and scoped to the most useful `devenv` Go options.

## Scope

This design supports only:

- `version`
- `package`

This design explicitly does not support:

- `enableHardeningWorkaround`
- `lsp.enable`
- `lsp.package`
- writing explicit `null` values into generated config

## Why This Scope

The official Go module exposes a small surface area, but only `version` and `package` are required for a useful first pass. `version` is especially important because it relies on `go-overlay`, so the implementation must coordinate both `devenv.nix` and `devenv.yaml`.

Keeping the first iteration small preserves the same delivery pattern already used for Rust and Python: a tight prompt flow, explicit rendering behavior, and focused tests.

## Current Project Structure

The existing language initialization path already has the required extension points:

- `src/main.rs` selects the language and dispatches to language-specific prompting
- `src/prompt.rs` collects interactive config
- `src/schema/context.rs` defines the serializable language model
- `templates/devenv.nix.tera` renders per-language `devenv.nix`
- `templates/devenv.yaml.tera` renders extra devenv inputs such as overlays
- `tests/generator_tests.rs` covers template rendering
- `tests/generate_rust_file_tests.rs` covers file planning behavior

Go support should follow that existing path without introducing extra abstractions.

## Reference Behavior

According to the official docs:

- `languages.go.package` defaults to `pkgs.go`
- `languages.go.version` automatically sets `languages.go.package` using `go-overlay`

That means:

- explicitly configured `package` can be rendered as a raw Nix expression
- explicitly configured `version` requires `go-overlay` to be present in `devenv.yaml`

## Data Model

`Language::Go` will become a structured enum variant with:

- `version: Option<String>`
- `package: Option<String>`

Rationale:

- matches the existing optional-field modeling used by Rust and Python
- omits unset fields from serialized template context
- keeps generated output limited to user-selected values

## Prompt Flow

Add `prompt_go_config()` in `src/prompt.rs`.

Flow:

1. Ask `use default go config?`
2. If yes, return Go config with all optional fields unset
3. If no, prompt for:
   - `version` as optional text
   - `package` as optional text, allowing raw Nix expressions like `pkgs.go_1_24`

Prompt rules:

- empty input maps to `None`
- no mutual-exclusion enforcement is added in the first version
- if both are set, both are rendered; this keeps behavior explicit and simple

## Rendering

`templates/devenv.nix.tera` gains a Go branch:

```nix
languages.go = {
  enable = true;
  version = "1.22.0";
  package = pkgs.go_1_24;
};
```

Rendering rules:

- always emit `enable = true;`
- emit `version` only when set
- emit `package` only when set
- render `package` as raw Nix syntax, not a quoted string

`templates/devenv.yaml.tera` gains conditional `go-overlay` support:

- if the selected language is Go and `version` is set, render:

```yaml
  go-overlay:
    url: github:nix-community/go-overlay
    inputs:
      nixpkgs:
        follows: nixpkgs
```

- if Go is selected with only `package`, do not add the overlay

This behavior is an implementation of the official docs statement that `version` uses `go-overlay`.

## CLI Integration

`src/main.rs` will import `prompt_go_config()` and use it in the `LanguageChoice::Go` branch.

Rust and Python behavior remain unchanged.

## Testing Strategy

Follow TDD and extend the current integration-style tests.

Add rendering coverage in `tests/generator_tests.rs` for:

- default Go output
- Go with `version`
- Go with `package`
- Go `devenv.yaml` with `version` includes `go-overlay`
- Go `devenv.yaml` with only `package` does not include `go-overlay`

Add planning coverage in `tests/generate_rust_file_tests.rs` for:

- a Go project still generates `devenv.nix`, `devenv.yaml`, and `.envrc`
- generated `devenv.nix` contains `languages.go`

## Risks

- invalid Nix expressions in `package` will produce invalid Nix output
- rendering both `version` and `package` may be redundant, but it is acceptable for the first version
- overlay behavior must stay tied to `version`, not `package`

## Non-Goals

- validating Nix expressions
- implementing all Go language module options
- invoking `devenv` or verifying overlay behavior against a live shell session
