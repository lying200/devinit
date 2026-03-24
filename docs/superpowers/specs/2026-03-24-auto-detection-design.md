# Auto Detection Design

## Goal

Add a first project auto-detection flow that can infer a single primary language and a small set of useful configuration fields, then let the user either accept the detected result or fall back to the existing manual prompt flow.

The first implementation is primarily about architecture and control flow. It must be easy to extend with more languages and more nuanced detection later without turning `src/main.rs` into a large decision tree.

## Product Rules

The detection behavior is intentionally conservative.

- if `--lang` is provided, skip auto detection entirely
- `--lang` means the user wants full control of language selection
- if `--lang` is not provided, run auto detection first
- if detection finds a primary language and useful config, show the result and ask for confirmation
- if the user accepts, generate files from the detected result
- if the user rejects, fall back to the current interactive manual flow
- if detection finds nothing, fall back directly to the current interactive manual flow

The first version explicitly does not support:

- merging detected config with manual edits
- showing multiple language candidates to the user
- partial correction of detected fields
- multi-language project configuration

## Why This Shape

The current project already has a working interactive flow:

- preflight checks
- language selection
- language-specific prompting
- `ProjectContext` creation
- template rendering
- optional Git ignore handling

That flow works well for explicit user choice, but the main value of `devinit` is to reduce setup work. Project auto detection is therefore the core feature direction.

At the same time, the project currently models each language directly in `Language` and keeps the main CLI orchestration in `src/main.rs`. If detection logic is added directly to the existing main flow, future support for more languages and more heuristics will become hard to maintain.

The design should therefore introduce a dedicated detection layer now, but stop short of a full plugin registry. That gives the project a clean architecture for near-term implementation while preserving a smooth migration path toward a more pluggable detector system later.

## Current Project Structure

The current language initialization path is:

- `src/main.rs` handles the end-to-end CLI flow
- `src/cli.rs` defines explicit CLI language selection through `--lang`
- `src/prompt.rs` contains the manual interactive prompt logic
- `src/schema/context.rs` defines the serializable `Language` enum used by rendering
- `src/generator.rs` and `src/generator/engine.rs` render `ProjectContext` into files
- `src/init_guard.rs` handles preflight environment detection
- `src/git_ignore.rs` handles post-generation Git ignore logic

This means the right place for new work is a new detection layer between the preflight checks and the existing manual language prompt path.

## Recommended Architecture

Introduce a new detection module with the following file layout:

- `src/detection.rs`
- `src/detection/types.rs`
- `src/detection/engine.rs`
- `src/detection/detectors.rs`
- `src/detection/detectors/rust.rs`
- `src/detection/detectors/python.rs`
- `src/detection/detectors/go.rs`
- `src/detection/detectors/java.rs`
- `src/detection/detectors/javascript.rs`

Purpose of each unit:

- `src/detection.rs`
  - public entry point for detection
  - exposes a high-level function such as `detect_project(...)`
- `src/detection/types.rs`
  - pure data structures for detection results
- `src/detection/engine.rs`
  - orchestrates detector execution
  - selects the primary language from candidates
- `src/detection/detectors.rs`
  - declares detector submodules
  - exposes the detector list to the engine
- `src/detection/detectors/*.rs`
  - language-specific scanning and parsing logic

This keeps the detection flow modular today and close to a detector-registry architecture tomorrow, without introducing extra abstraction too early.

## Detection Result Model

The first version should produce a normalized detection result that can be consumed directly by the existing generation flow.

Recommended types:

- `DetectionOutcome`
  - `NoMatch`
  - `Match { candidate: LanguageCandidate }`

- `LanguageCandidate`
  - `language: Language`
  - `confidence: DetectionConfidence`
  - `reasons: Vec<String>`

- `DetectionConfidence`
  - `High`
  - `Medium`
  - `Low`

Important modeling rule:

- detection should produce a `Language` value directly

Why:

- the generator already consumes `Language`
- the first version does not support partial editing of detected config
- a second parallel configuration model would add complexity before it is needed

This does not block future evolution. If later work needs a distinction between raw detection output and final user-approved config, that layer can be introduced above the existing result model.

## Detector Interface

The first version should not introduce a full trait-based registry yet, but every detector should still follow the same boundary:

- input: `&Path`
- output: `io::Result<Option<LanguageCandidate>>`

This gives the project a stable internal contract:

- each detector can be tested independently
- `engine` only depends on detector outputs
- migrating later to a registry or trait object model becomes a structural change rather than a behavior rewrite

The engine must not parse language-specific files itself. For example:

- Rust parsing belongs only in the Rust detector
- JavaScript parsing belongs only in the JavaScript detector
- `engine` only aggregates and chooses

## Main Flow Integration

The updated CLI flow becomes:

1. parse CLI args
2. verify target directory exists
3. run existing environment guard
4. resolve language configuration:
   - if `--lang` is provided, use the existing manual path for that explicit language
   - otherwise, run auto detection
5. if detection returns a match:
   - print a concise summary of the detected language and fields
   - ask the user whether to use the detected config
   - if yes, continue with generation
   - if no, fall back to the existing manual flow
6. if detection returns no match:
   - use the existing manual flow
7. build `ProjectContext`
8. render and write files
9. run the existing Git ignore flow

This keeps file generation and Git handling unchanged. Detection only affects how the final `Language` value is resolved.

## Manual vs Detected Resolution

The main flow should be refactored so that `src/main.rs` depends on one higher-level resolution step rather than branching repeatedly on CLI and prompt details.

Recommended shape:

- `resolve_language_config(target_dir, cli_lang) -> io::Result<Language>`

This function owns the decision tree:

- explicit `--lang`
- auto detection
- user confirmation
- fallback to manual prompts

With this shape:

- `main.rs` stays focused on orchestration
- later work can add partial correction or richer candidate selection without rewriting the whole CLI flow

## Primary Language Selection

The first version should choose only one primary language.

Rationale:

- minimizes user interaction
- keeps generated config simple
- avoids early complexity around mixed-language workspaces

Selection should use:

- strong file signals
- then a fixed priority order

The first version should not use a sophisticated score model. The decision logic should be explainable and stable.

Recommended strong signals:

- Rust: `Cargo.toml`
- Python: `pyproject.toml`, then `requirements.txt`
- Go: `go.mod`
- Java: `pom.xml`, `build.gradle`, `build.gradle.kts`
- JavaScript: `package.json`

Recommended priority order for collisions:

1. Rust
2. Python
3. Go
4. Java
5. JavaScript

This priority is not a statement about language importance. It is a practical choice for the first version:

- `Cargo.toml` and `go.mod` are usually strong root-level ownership signals
- `package.json` is common in mixed repositories and should not dominate early

The engine should record the reasons that led to the final choice, but only expose the final primary candidate in the first version.

## V1 Detection Scope By Language

The first version should detect only high-value, low-risk fields.

### Rust

Signals:

- `Cargo.toml`
- `rust-toolchain.toml`
- `rust-toolchain`

Config to infer:

- language = Rust
- version or channel only when toolchain files provide a clear value

Do not attempt to infer components or targets in the first version.

### Python

Signals:

- `pyproject.toml`
- `requirements.txt`
- `.python-version`

Config to infer:

- language = Python
- version from `.python-version` when present and simple

The first version should treat `pyproject.toml` mainly as a Python project signal, not a source of complex version-constraint parsing.

Do not infer `uv` or `venv` yet unless a later implementation introduces a simple and clearly justified rule.

### Go

Signals:

- `go.mod`

Config to infer:

- language = Go
- version from the `go` directive when present

### Java

Signals:

- `pom.xml`
- `build.gradle`
- `build.gradle.kts`

Config to infer:

- language = Java
- `maven_enable = true` when `pom.xml` is present
- `gradle_enable = true` when Gradle build files are present

The first version should not try to infer JDK version unless a later implementation adds a stable and simple source for it.

### JavaScript

Signals:

- `package.json`

Config to infer:

- language = JavaScript
- package manager from `packageManager` when clearly declared

Optional version inference:

- use `engines.node` only if the format is simple and directly mappable

If the field is ambiguous, do not infer a version.

## Confirmation UX

Detection and confirmation should stay separate.

Detection only decides:

- whether a project was recognized
- what the candidate language config is
- what reasons support the guess

The CLI interaction layer decides whether the user accepts that result.

Suggested first-version interaction:

- if no match, go directly to manual flow
- if match, show:
  - detected language
  - detected key fields
  - a short list of reasons
- then ask a single confirmation question such as:
  - `Use detected config?`

If the user says:

- yes: continue with the detected `Language`
- no: fall back to the existing manual flow

The first version should not:

- show multiple candidate languages
- show detailed scoring
- offer field-by-field edits

## Extensibility Toward a Registry Model

The design should make a later migration toward a fuller detector system straightforward.

To support that, the first implementation should preserve these boundaries:

- each detector lives in its own file
- each detector obeys one function signature
- `engine` is the only place that knows how to iterate across detectors
- `main.rs` only depends on a high-level language-resolution function
- detection results always include explanation strings

If the project later moves to a registry or trait model, those same concepts can stay in place:

- detector files become detector implementations
- the detector list becomes registration
- the resolution flow remains unchanged

## Testing Strategy

The first implementation should be driven by tests in layers.

### Detector Tests

Add focused tests for each language detector:

- returns `None` when the key signal is absent
- returns the correct language when the key signal exists
- infers supported fields only when the source is clear
- leaves unsupported or ambiguous fields unset

These should be direct module tests whenever possible.

### Engine Tests

Add tests for:

- no candidates -> `NoMatch`
- one candidate -> selected
- multiple candidates -> fixed priority winner
- reasons from the winning candidate are preserved

### CLI Flow Tests

Add integration-style coverage for:

- explicit `--lang` skips detection
- no `--lang` with no match falls back to manual flow
- no `--lang` with detected match prompts for confirmation
- accepted detection uses detected config
- rejected detection falls back to manual flow

The first version does not need to test every detector through the whole CLI if that would make the suite too heavy. Detector behavior and CLI orchestration should be validated separately where practical.

## Risks

- ambiguous repositories may still choose a primary language that is not what the user wants
- simple fixed-priority selection may feel wrong in some polyglot repositories
- direct parsing of project files can grow complicated if the first version tries to infer too many fields
- CLI tests for interactive confirmation can become brittle if prompt structure is tightly coupled to output text

## Non-Goals

- multi-language output generation
- partial editing of detected config
- detector scoring UI
- invoking package managers or language toolchains
- validating every inferred version against upstream tooling
- replacing the current manual prompt flow
