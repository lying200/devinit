# devinit

Automatically detect project languages and generate [devenv](https://devenv.sh) development environment configuration.

[中文文档](README.md)

## Features

- 5 languages supported: Rust, Python, Go, Java, JavaScript
- Auto-detection: language identification + version inference
- Monorepo support: scans immediate subdirectories
- Multi-language: configure multiple languages in a single project
- Partial modification: keep correct detections, override the rest
- Non-interactive mode: `--yes` skips prompts, suitable for CI/CD
- `--force` to overwrite existing configuration
- Existing devenv/direnv/Nix environment protection
- Git ignore handling (`.gitignore` or `.git/info/exclude`)

## Installation

### Nix Flake

```bash
# Run directly
nix run github:lying200/devinit

# Install to profile
nix profile install github:lying200/devinit

# Reference in flake.nix
{
  inputs.devinit.url = "github:lying200/devinit";
}
```

### Build from source

```bash
cargo install --path .
```

## Usage

```bash
# Interactive (auto-detect languages, prompt for confirmation)
devinit

# Specify languages
devinit --lang go
devinit --lang go,javascript

# Non-interactive (auto-detect + defaults, for CI/CD)
devinit --yes

# Non-interactive + specify languages
devinit --yes --lang go,javascript

# Overwrite existing configuration
devinit --force

# Specify target directory
devinit /path/to/project
```

## Generated files

| File | Description |
|------|-------------|
| `devenv.nix` | Language toolchains, packages, environment variables |
| `devenv.yaml` | devenv inputs (nixpkgs + language overlays) |
| `.envrc` | direnv integration to activate the devenv environment |

## License

[MIT](LICENSE)
