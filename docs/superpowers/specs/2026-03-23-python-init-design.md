# Python Init Design

## Goal

Add a first minimal Python initialization flow to the CLI, modeled after the existing Rust flow and constrained to the most common `devenv` Python options.

## Scope

This design adds support for these Python fields only:

- `version`
- `package`
- `uv.enable`
- `venv.enable`
- `venv.quiet`

This design explicitly does not support:

- `venv.requirements`
- `poetry.*`
- `uv.sync.*`
- Python language server / lint / formatter integrations
- Writing explicit `null` defaults back into generated config

## Why This Scope

`devenv` exposes many Python options, but the first deliverable should stay small and predictable. The selected fields map cleanly to common Python project setup while keeping the prompt flow simple and the generated Nix readable.

`venv.requirements` is intentionally excluded because it triggers shell-time package installation behavior and is heavier than the rest of the chosen fields. The initial implementation should avoid that side effect.

## Current Project Structure

The existing Rust implementation already defines the shape of the feature:

- `src/main.rs` selects the language and builds `ProjectContext`
- `src/prompt.rs` collects language-specific interactive configuration
- `src/schema/context.rs` defines the serializable language model consumed by templates
- `templates/devenv.nix.tera` renders language-specific Nix config
- `templates/devenv.yaml.tera` renders extra devenv inputs when required
- `tests/generator_tests.rs` verifies template rendering
- `tests/generate_rust_file_tests.rs` verifies file planning behavior

Python support should follow the same path and not introduce a new abstraction layer yet.

## Data Model

`Language::Python` will become a structured enum variant instead of a bare marker variant.

Fields:

- `version: Option<String>`
- `package: Option<String>`
- `uv_enable: Option<bool>`
- `venv_enable: Option<bool>`
- `venv_quiet: Option<bool>`

Rationale:

- `Option<T>` matches the existing Rust modeling style
- omitted values remain omitted in templates
- generated output stays close to what the user explicitly selected

Constraint:

- `venv_quiet` is only meaningful when `venv_enable` is enabled in the prompt flow

## Prompt Flow

Add `prompt_python_config()` in `src/prompt.rs`.

Flow:

1. Ask `use default python config?`
2. If yes, return a Python config with all optional fields unset
3. If no, prompt in order:
   - `version` as optional text
   - `package` as optional text, allowing raw Nix expressions such as `pkgs.python311`
   - `enable uv?` as boolean
   - `enable venv?` as boolean
   - if `venv` is enabled, ask `quiet?`

Prompt design rules:

- empty text input maps to `None`
- booleans are only emitted when the user explicitly chooses a value in the non-default path
- no prompt is added for `venv.requirements`

## Rendering

`templates/devenv.nix.tera` will gain a Python branch:

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

Rendering rules:

- always emit `enable = true;` for the selected language
- emit `version` only when set
- emit `package` as a raw Nix expression, not a quoted string
- emit `uv.enable` only when set
- emit `venv.enable` only when set
- emit `venv.quiet` only when set
- do not emit any field with an unset value

`templates/devenv.yaml.tera` remains unchanged because Python does not need the Rust overlay pattern.

## CLI Integration

`src/main.rs` will stop mapping Python to a bare `Language::Python` value and instead call `prompt_python_config()`.

Rust behavior remains unchanged.

## Testing Strategy

Follow TDD and extend the existing test style rather than adding new harnesses.

Add rendering coverage in `tests/generator_tests.rs` for:

- default Python output
- Python with `version`
- Python with `package`
- Python with `uv.enable`
- Python with `venv.enable` and `venv.quiet`

Add file-planning coverage in `tests/generate_rust_file_tests.rs` or a neighboring test file for:

- a Python project still generates `devenv.nix`, `devenv.yaml`, and `.envrc`
- the generated Nix content includes `languages.python`

## Risks

- `package` is rendered as raw Nix syntax, so invalid user input will generate invalid Nix
- adding more Python options later may make the current enum variant large; acceptable for this stage
- prompt branching for `venv_quiet` must avoid producing inconsistent states

## Non-Goals

- validating Nix expressions
- invoking `devenv`, `pip`, or networked package installation
- supporting all options from the Python module in one pass
