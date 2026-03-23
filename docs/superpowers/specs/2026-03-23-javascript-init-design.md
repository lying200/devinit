# JavaScript Init Design

## Goal

Replace the project's legacy `Nodejs` naming with `JavaScript`, matching devenv's official `languages.javascript.*` module, and add a first useful JavaScript initialization flow to the CLI.

## Scope

This design supports:

- renaming `Nodejs` references in the project to `JavaScript`
- `languages.javascript.package`
- `languages.javascript.npm.enable`
- `languages.javascript.pnpm.enable`
- `languages.javascript.yarn.enable`
- `languages.javascript.corepack.enable`
- `languages.javascript.bun.enable`

This design explicitly does not support:

- `bun.package`
- `bun.install.enable`
- `npm.package`
- `npm.install.enable`
- `pnpm.package`
- `pnpm.install.enable`
- `yarn.package`
- `yarn.install.enable`
- `directory`
- `lsp.enable`
- `lsp.package`

## Why This Scope

The current project still uses `Nodejs` as a language label, while devenv's official module is `languages.javascript`. Renaming now prevents future confusion and keeps the codebase aligned with the official documentation and option names.

For the first implementation, the most useful controls are the Node.js package plus the common package-manager toggles, with Bun included because some projects need it. The remaining options are more specialized and can be added later without invalidating this design.

## Current Project Structure

The existing language-init path is already consistent across Rust, Python, Go, and Java:

- `src/cli.rs` defines the user-selectable language enum
- `src/main.rs` dispatches to the appropriate language prompt
- `src/prompt.rs` collects interactive configuration
- `src/schema/context.rs` defines the serializable language model
- `templates/devenv.nix.tera` renders language configuration
- `templates/devenv.yaml.tera` renders extra inputs when required
- `tests/generator_tests.rs` validates rendering
- `tests/generate_rust_file_tests.rs` validates file planning

JavaScript should follow the same path and not introduce a new abstraction layer.

## Reference Behavior

According to the official docs:

- `languages.javascript.enable` enables JavaScript tooling
- `languages.javascript.package` selects the Node.js package and defaults to `pkgs.nodejs-slim`
- `npm.enable`, `pnpm.enable`, `yarn.enable`, `corepack.enable`, and `bun.enable` are booleans

There is no indication in the selected scope that extra `devenv.yaml` inputs are required, unlike Python and Go version handling.

## Naming Strategy

The project will stop using `Nodejs` and adopt `JavaScript` terminology throughout the codebase.

This affects:

- `LanguageChoice::Nodejs` -> `LanguageChoice::JavaScript`
- `Language::Nodejs` -> `Language::JavaScript`
- interactive menu labels: `"Nodejs"` -> `"JavaScript"`

The generated devenv config will use `languages.javascript`, which matches the official module name.

## Data Model

`Language::JavaScript` will be a structured enum variant with:

- `package: Option<String>`
- `npm_enable: Option<bool>`
- `pnpm_enable: Option<bool>`
- `yarn_enable: Option<bool>`
- `corepack_enable: Option<bool>`
- `bun_enable: Option<bool>`

Rationale:

- matches the optional-field modeling used by the existing language variants
- omits unset fields from generated config
- keeps generated output limited to explicitly selected options

## Prompt Flow

Add `prompt_javascript_config()` in `src/prompt.rs`.

Flow:

1. Ask `use default javascript config?`
2. If yes, return JavaScript config with all optional fields unset
3. If no, prompt for:
   - `package` as optional text, allowing raw Nix expressions like `pkgs.nodejs_22`
   - `enable npm?`
   - `enable pnpm?`
   - `enable yarn?`
   - `enable corepack?`
   - `enable bun?`

Prompt rules:

- empty text input maps to `None`
- booleans are only emitted on the non-default path
- no extra validation is added in the first version for package-manager combinations

## Rendering

`templates/devenv.nix.tera` gains a JavaScript branch:

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

Rendering rules:

- always emit `enable = true;`
- emit `package` only when set
- emit each `*.enable` field only when set
- render `package` as raw Nix syntax, not a quoted string

`templates/devenv.yaml.tera` remains unchanged for JavaScript because the selected scope does not require extra inputs.

## CLI Integration

`src/main.rs` will import and use `prompt_javascript_config()`.

`src/cli.rs` will rename the enum variant from `Nodejs` to `JavaScript`.

Other language behavior remains unchanged.

## Testing Strategy

Follow TDD and extend the current rendering and planning tests.

Add rendering coverage in `tests/generator_tests.rs` for:

- default JavaScript output
- JavaScript with `package`
- JavaScript with `npm.enable`
- JavaScript with `pnpm.enable`
- JavaScript with `yarn.enable`
- JavaScript with `corepack.enable`
- JavaScript with `bun.enable`
- JavaScript `devenv.yaml` remains the base input set

Add planning coverage in `tests/generate_rust_file_tests.rs` for:

- a JavaScript project still generates `devenv.nix`, `devenv.yaml`, and `.envrc`
- generated `devenv.nix` contains `languages.javascript`

## Risks

- invalid Nix expressions in `package` will produce invalid Nix output
- the first version does not guard against redundant package-manager combinations
- full `Nodejs` -> `JavaScript` renaming touches several files, so missed references would create inconsistent terminology

## Non-Goals

- exposing every JavaScript module option in one pass
- validating Nix expressions
- adding install-time toggles or project-directory handling
- invoking `devenv` to verify runtime behavior
