use devinit::generator::{
    render_devenv_nix,
    render_devenv_yaml,
    render_envrc,
};
use devinit::schema::{Language, ProjectContext};

fn nomalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[test]
fn test_render_rust_base() {
    let project_ctx = ProjectContext {
        language: Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: None,
        },
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
            pkgs.git
          ];

          languages.rust = {
            enable = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_rust_with_channel() {
    let project_ctx = ProjectContext {
        language: Language::Rust {
            channel: Some("stable".to_string()),
            version: None,
            components: None,
            targets: None,
        },
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
            pkgs.git
          ];

          languages.rust = {
            enable = true;
            channel = "stable";
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_rust_with_version() {
    let project_ctx = ProjectContext {
        language: Language::Rust {
            channel: None,
            version: Some("1.81.0".to_string()),
            components: None,
            targets: None,
        },
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
            pkgs.git
          ];

          languages.rust = {
            enable = true;
            version = "1.81.0";
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_rust_with_components() {
    let project_ctx = ProjectContext {
        language: Language::Rust {
            channel: None,
            version: None,
            components: Some(vec!["clippy".to_string()]),
            targets: None,
        },
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
            pkgs.git
          ];

          languages.rust = {
            enable = true;
            components = [
              "clippy"
            ];
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_rust_with_targets() {
    let project_ctx = ProjectContext {
        language: Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: Some(vec!["wasm32-unknown-unknown".to_string()]),
        },
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
            pkgs.git
          ];

          languages.rust = {
            enable = true;
            targets = [
              "wasm32-unknown-unknown"
            ];
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_devenv_yaml() {
    let project_ctx = ProjectContext {
        language: Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: None,
        },
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = r#"
        inputs:
          nixpkgs:
            url: github:cachix/devenv-nixpkgs/rolling
          rust-overlay:
            url: github:oxalica/rust-overlay
            inputs:
              nixpkgs:
                follows: nixpkgs
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_envrc() {
    let envrc = render_envrc();
    let expected = r#"
        #!/usr/bin/env bash
        
        eval "$(devenv direnvrc)"
        
        # You can pass flags to the devenv command
        # For example: use devenv --impure --option services.postgres.enable:bool true
        use devenv
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&envrc)
    )
}
