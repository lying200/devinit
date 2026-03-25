# Devinit Nix Flake Packaging Design

## Goal

Allow `devinit` to be installed directly from its GitHub repository on a NixOS host through flakes, while also supporting `nix run` and `nix profile install` as first-class entry points.

The package design should stay self-contained inside this repository. It does not need to optimize yet for upstream `nixpkgs` inclusion.

## Scope

This design supports:

- adding a repository-level `flake.nix`
- moving package build logic into a dedicated `nix/package.nix`
- exposing `packages`, `apps`, and `devShells` outputs
- building the Rust CLI from the checked-in `Cargo.lock`
- making `git` available at runtime inside the packaged CLI
- documenting NixOS, `nix run`, and `nix profile install` usage in the README

This design explicitly does not support:

- overlays
- `checks` outputs beyond what is needed for local verification
- non-flake installation paths
- preparing the repository for direct upstream import into `nixpkgs`
- broad multi-platform validation beyond Linux

## Why This Scope

The current project is a small Rust CLI intended to be used interactively by developers. The main distribution problem is not the Rust build itself, but turning the repository into something a NixOS machine can consume cleanly from GitHub.

The repository already contains `devenv.nix` for local development, but that does not make the project installable as a reusable package. A consumer host needs a stable flake output shape that can be referenced from its own system flake.

At the same time, pushing all package logic into `flake.nix` would make the first packaging step harder to maintain. The design should therefore keep the flake thin and place the actual build expression in a dedicated package file.

## Current Project Structure

The current repository shape relevant to packaging is:

- `Cargo.toml` defines a single Rust package named `devinit`
- `Cargo.lock` is present and should remain the source of dependency locking for Nix builds
- `src/main.rs` defines a single CLI binary
- `src/git_ignore.rs` and the integration tests call `git` as an external command
- `README.md` does not yet describe any Nix installation path
- `devenv.nix` exists only for contributor development, not for consumer installation

This means the package boundary is straightforward:

- one Rust binary
- one runtime external dependency on `git`
- one repository root suitable to expose as a flake

## Approaches Considered

### Recommended: `flake.nix` plus `nix/package.nix`

Expose user-facing outputs from `flake.nix`, but keep the Rust packaging logic in `nix/package.nix`.

Why this is recommended:

- keeps the flake readable
- gives the package logic a stable place to evolve
- cleanly separates output wiring from build details
- is enough structure for a self-contained repository without over-design

### Alternative: put everything directly in `flake.nix`

Why this is not recommended:

- fast to start, but package logic and output wiring become mixed together
- makes future packaging changes noisier than necessary

### Alternative: design immediately for `nixpkgs` inclusion

Why this is not recommended:

- adds structure that the current user goal does not require
- increases review and maintenance cost without improving current install ergonomics

## Output Shape

The repository should expose the following flake outputs per supported system:

- `packages.${system}.default`
- `packages.${system}.devinit`
- `apps.${system}.default`
- `devShells.${system}.default`

Behavioral intent:

- `packages.default` is the canonical package output
- `packages.devinit` is a named alias for readability
- `apps.default` points to the packaged `devinit` binary so `nix run` works naturally
- `devShells.default` provides a minimal contributor shell for local packaging and Rust work

No overlay output is needed in the first version.

## Supported Systems

The first version should explicitly target only:

- `x86_64-linux`
- `aarch64-linux`

Why:

- the immediate use case is installation onto NixOS hosts
- this keeps the validation surface narrow
- Darwin support can be added later without changing the package model

## Package Definition

`nix/package.nix` should define the package as a function over `pkgs`-level inputs and use `rustPlatform.buildRustPackage`.

Recommended behavior:

- package name should remain `devinit`
- source should be the repository root
- `Cargo.lock` should be used directly for dependency locking
- metadata should stay minimal and accurate rather than speculative

This should produce one installable binary in `$out/bin/devinit`.

## Runtime Dependency Handling

The packaged CLI must make `git` available at runtime.

Reason:

- `devinit` invokes `git` through `std::process::Command`
- a Nix-installed executable cannot assume `/usr/bin/git` or user-global PATH contents
- without explicit runtime wiring, some CLI paths will fail even though the package builds successfully

Recommended approach:

- add `git` as a runtime dependency
- wrap the installed binary so the runtime PATH includes the packaged `git`

This keeps the Rust source unchanged for the first version and makes behavior consistent across:

- `nix run`
- `nix profile install`
- NixOS `environment.systemPackages`

## Flake Wiring

`flake.nix` should:

- import `nixpkgs`
- use a small per-system loop such as `flake-utils.lib.eachSystem`
- instantiate `pkgs` for each supported Linux system
- call `nix/package.nix`
- expose the package under both `default` and `devinit`
- define `apps.default.program` from the packaged binary path
- define a minimal `devShell` with Rust tooling and Nix packaging helpers appropriate to current repository conventions

The flake should stay focused on package exposure. It should not duplicate the package build logic inline.

## Consumer Experience

The intended user-facing flows are:

### NixOS system installation

The consuming host adds this repository as a flake input and installs:

- `inputs.devinit.packages.${system}.default`

This is the primary target flow and should be shown in the README.

### Ad-hoc execution

Users can run:

- `nix run github:<owner>/devinit`

This should invoke the default app and execute the packaged binary directly.

### User profile installation

Users can install:

- `nix profile install github:<owner>/devinit`

This should resolve to the default package output.

## README Changes

`README.md` should gain a short Nix installation section that documents:

- `nix run github:<owner>/devinit`
- `nix profile install github:<owner>/devinit`
- a minimal NixOS flake snippet using this repository as an input

The documentation should stay concrete and avoid introducing non-flake paths or speculative platform guarantees.

## Verification Strategy

The implementation should be verified at the package boundary.

Minimum verification commands:

- `nix build .#devinit`
- `nix run . -- --help`

If local environment constraints allow, also verify:

- `nix profile install .`

The verification focus is:

- the package builds from the repository root
- the default app launches the CLI
- the binary is exported under the expected name
- runtime `git` wiring is present

## Risks

- runtime dependency issues may only appear after installation if `git` is not wrapped correctly
- keeping Linux-only support in the first version means non-NixOS users may assume unsupported platforms work when they have not been validated
- README examples become part of the public interface and should match the actual flake outputs exactly

## Non-Goals

- refactoring Rust code to remove external `git` calls
- adding binary caches or release automation
- exposing overlays or module outputs
- making the package acceptable for immediate upstreaming into `nixpkgs`
