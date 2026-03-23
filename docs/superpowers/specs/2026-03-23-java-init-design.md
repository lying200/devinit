# Java Init Design

## Goal

Add a first minimal Java initialization flow to the CLI, aligned with the existing Rust, Python, and Go flows and limited to the most common `devenv` Java options.

## Scope

This design supports only:

- `jdk.package`
- `gradle.enable`
- `maven.enable`

This design explicitly does not support:

- `gradle.package`
- `maven.package`
- `lsp.enable`
- `lsp.package`
- writing explicit `null` values into generated config

## Why This Scope

The Java module exposes more options than Go, but the first pass should stay small. A custom JDK plus simple Gradle and Maven toggles are enough to make the feature useful while preserving the lightweight CLI interaction style already established in the project.

The Gradle and Maven package options are intentionally excluded because the module already defaults them based on the selected JDK. The first version should rely on those built-in defaults rather than exposing more raw package inputs.

## Current Project Structure

The current language initialization flow already has clear extension points:

- `src/main.rs` selects the target language and dispatches to language-specific prompt functions
- `src/prompt.rs` collects interactive configuration
- `src/schema/context.rs` defines the serializable language model
- `templates/devenv.nix.tera` renders per-language `devenv.nix`
- `templates/devenv.yaml.tera` renders extra inputs when required
- `tests/generator_tests.rs` verifies rendering behavior
- `tests/generate_rust_file_tests.rs` verifies file planning behavior

Java support should follow the same path without introducing a new abstraction layer.

## Reference Behavior

According to the official docs:

- `languages.java.jdk.package` defaults to `pkgs.jdk`
- `languages.java.gradle.enable` defaults to `false`
- `languages.java.maven.enable` defaults to `false`
- the default Gradle and Maven packages inherit the configured JDK

This means:

- a custom `jdk.package` can be rendered as a raw Nix expression
- enabling Gradle or Maven only requires boolean toggles in the first version
- no additional `devenv.yaml` input is needed for the selected scope

## Data Model

`Language::Java` will become a structured enum variant with:

- `jdk_package: Option<String>`
- `gradle_enable: Option<bool>`
- `maven_enable: Option<bool>`

Rationale:

- matches the optional-field modeling used by Rust, Python, and Go
- omits unset fields from the template context
- keeps generated output limited to user-selected values

## Prompt Flow

Add `prompt_java_config()` in `src/prompt.rs`.

Flow:

1. Ask `use default java config?`
2. If yes, return Java config with all optional fields unset
3. If no, prompt for:
   - `jdk package` as optional text, allowing raw Nix expressions like `pkgs.jdk17`
   - `enable gradle?` as boolean
   - `enable maven?` as boolean

Prompt rules:

- empty text input maps to `None`
- booleans are stored as optional values and emitted only on the non-default path
- no prompt is added for `lsp` or custom Gradle/Maven packages

## Rendering

`templates/devenv.nix.tera` gains a Java branch:

```nix
languages.java = {
  enable = true;
  jdk.package = pkgs.jdk17;
  gradle.enable = true;
  maven.enable = true;
};
```

Rendering rules:

- always emit `enable = true;`
- emit `jdk.package` only when set
- emit `gradle.enable` only when set
- emit `maven.enable` only when set
- render `jdk.package` as raw Nix syntax, not a quoted string

`templates/devenv.yaml.tera` remains unchanged for Java because the selected scope does not require extra inputs or overlays.

## CLI Integration

`src/main.rs` will import `prompt_java_config()` and use it in the `LanguageChoice::Java` branch.

Existing Rust, Python, and Go behavior remain unchanged.

## Testing Strategy

Follow TDD and extend the current rendering and planning tests.

Add rendering coverage in `tests/generator_tests.rs` for:

- default Java output
- Java with `jdk.package`
- Java with `gradle.enable`
- Java with `maven.enable`
- Java `devenv.yaml` remains the base input set

Add planning coverage in `tests/generate_rust_file_tests.rs` for:

- a Java project still generates `devenv.nix`, `devenv.yaml`, and `.envrc`
- generated `devenv.nix` contains `languages.java`

## Risks

- invalid Nix expressions in `jdk.package` will produce invalid Nix output
- the first version does not expose `lsp.enable`, so users rely on module defaults
- later support for `gradle.package` and `maven.package` may expand the enum variant, which is acceptable at this stage

## Non-Goals

- validating Nix expressions
- exposing every Java module option in one pass
- invoking `devenv` or verifying Java tool installation in a live shell
