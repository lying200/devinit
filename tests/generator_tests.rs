use devinit::generator::{render_devenv_nix, render_devenv_yaml, render_envrc};
use devinit::schema::{Language, ProjectContext};

fn nomalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[test]
fn test_render_rust_base_without_tools_has_no_git_package() {
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
    let devenv_conf = render_devenv_nix(&project_ctx);

    assert!(!devenv_conf.contains("pkgs.git"));
    assert!(devenv_conf.contains("packages = ["));
    assert!(devenv_conf.contains("languages.rust"));
}

#[test]
fn test_render_rust_with_explicit_git_tool_includes_git_package() {
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

    assert!(devenv_conf.contains("pkgs.git"));
    assert!(devenv_conf.contains("languages.rust"));
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
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
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
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
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
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
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
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
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
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
          packages = [
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
fn test_render_python_base() {
    let project_ctx = ProjectContext {
        language: Language::Python {
            version: None,
            package: None,
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
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

          languages.python = {
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
fn test_render_python_with_version() {
    let project_ctx = ProjectContext {
        language: Language::Python {
            version: Some("3.11".to_string()),
            package: None,
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
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

          languages.python = {
            enable = true;
            version = "3.11";
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_python_with_package() {
    let project_ctx = ProjectContext {
        language: Language::Python {
            version: None,
            package: Some("pkgs.python311".to_string()),
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
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

          languages.python = {
            enable = true;
            package = pkgs.python311;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_python_with_uv() {
    let project_ctx = ProjectContext {
        language: Language::Python {
            version: None,
            package: None,
            uv_enable: Some(true),
            venv_enable: None,
            venv_quiet: None,
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

          languages.python = {
            enable = true;
            uv.enable = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_python_with_venv() {
    let project_ctx = ProjectContext {
        language: Language::Python {
            version: None,
            package: None,
            uv_enable: None,
            venv_enable: Some(true),
            venv_quiet: Some(true),
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

          languages.python = {
            enable = true;
            venv.enable = true;
            venv.quiet = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_devenv_yaml_for_python_with_version() {
    let project_ctx = ProjectContext {
        language: Language::Python {
            version: Some("3.11".to_string()),
            package: None,
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        },
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = r#"
        inputs:
          nixpkgs:
            url: github:cachix/devenv-nixpkgs/rolling
          nixpkgs-python:
            url: github:cachix/nixpkgs-python
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
fn test_render_devenv_yaml_for_python_with_package_only() {
    let project_ctx = ProjectContext {
        language: Language::Python {
            version: None,
            package: Some("pkgs.python311".to_string()),
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        },
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = r#"
        inputs:
          nixpkgs:
            url: github:cachix/devenv-nixpkgs/rolling
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_go_base() {
    let project_ctx = ProjectContext {
        language: Language::Go {
            version: None,
            package: None,
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

          languages.go = {
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
fn test_render_go_with_version() {
    let project_ctx = ProjectContext {
        language: Language::Go {
            version: Some("1.22.0".to_string()),
            package: None,
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

          languages.go = {
            enable = true;
            version = "1.22.0";
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_go_with_package() {
    let project_ctx = ProjectContext {
        language: Language::Go {
            version: None,
            package: Some("pkgs.go_1_24".to_string()),
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

          languages.go = {
            enable = true;
            package = pkgs.go_1_24;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_devenv_yaml_for_go_with_version() {
    let project_ctx = ProjectContext {
        language: Language::Go {
            version: Some("1.22.0".to_string()),
            package: None,
        },
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = r#"
        inputs:
          nixpkgs:
            url: github:cachix/devenv-nixpkgs/rolling
          go-overlay:
            url: github:nix-community/go-overlay
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
fn test_render_devenv_yaml_for_go_with_package_only() {
    let project_ctx = ProjectContext {
        language: Language::Go {
            version: None,
            package: Some("pkgs.go_1_24".to_string()),
        },
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = r#"
        inputs:
          nixpkgs:
            url: github:cachix/devenv-nixpkgs/rolling
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_java_base() {
    let project_ctx = ProjectContext {
        language: Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: None,
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

          languages.java = {
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
fn test_render_java_with_jdk_package() {
    let project_ctx = ProjectContext {
        language: Language::Java {
            jdk_package: Some("pkgs.jdk17".to_string()),
            gradle_enable: None,
            maven_enable: None,
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

          languages.java = {
            enable = true;
            jdk.package = pkgs.jdk17;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_java_with_gradle() {
    let project_ctx = ProjectContext {
        language: Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
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

          languages.java = {
            enable = true;
            gradle.enable = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_java_with_maven() {
    let project_ctx = ProjectContext {
        language: Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: Some(true),
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

          languages.java = {
            enable = true;
            maven.enable = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_devenv_yaml_for_java() {
    let project_ctx = ProjectContext {
        language: Language::Java {
            jdk_package: Some("pkgs.jdk17".to_string()),
            gradle_enable: Some(true),
            maven_enable: Some(true),
        },
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = r#"
        inputs:
          nixpkgs:
            url: github:cachix/devenv-nixpkgs/rolling
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_javascript_base() {
    let project_ctx = ProjectContext {
        language: Language::JavaScript {
            package: None,
            package_manager: None,
            corepack_enable: None,
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

          languages.javascript = {
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
fn test_render_javascript_with_package() {
    let project_ctx = ProjectContext {
        language: Language::JavaScript {
            package: Some("pkgs.nodejs_22".to_string()),
            package_manager: None,
            corepack_enable: None,
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

          languages.javascript = {
            enable = true;
            package = pkgs.nodejs_22;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_javascript_with_npm() {
    let project_ctx = ProjectContext {
        language: Language::JavaScript {
            package: None,
            package_manager: Some("npm".to_string()),
            corepack_enable: None,
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

          languages.javascript = {
            enable = true;
            npm.enable = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_javascript_with_pnpm() {
    let project_ctx = ProjectContext {
        language: Language::JavaScript {
            package: None,
            package_manager: Some("pnpm".to_string()),
            corepack_enable: None,
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

          languages.javascript = {
            enable = true;
            pnpm.enable = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_javascript_with_yarn() {
    let project_ctx = ProjectContext {
        language: Language::JavaScript {
            package: None,
            package_manager: Some("yarn".to_string()),
            corepack_enable: None,
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

          languages.javascript = {
            enable = true;
            yarn.enable = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_javascript_with_corepack() {
    let project_ctx = ProjectContext {
        language: Language::JavaScript {
            package: None,
            package_manager: None,
            corepack_enable: Some(true),
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

          languages.javascript = {
            enable = true;
            corepack.enable = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_javascript_with_bun() {
    let project_ctx = ProjectContext {
        language: Language::JavaScript {
            package: None,
            package_manager: Some("bun".to_string()),
            corepack_enable: None,
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

          languages.javascript = {
            enable = true;
            bun.enable = true;
          };
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}

#[test]
fn test_render_devenv_yaml_for_javascript() {
    let project_ctx = ProjectContext {
        language: Language::JavaScript {
            package: Some("pkgs.nodejs_22".to_string()),
            package_manager: Some("pnpm".to_string()),
            corepack_enable: Some(true),
        },
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = r#"
        inputs:
          nixpkgs:
            url: github:cachix/devenv-nixpkgs/rolling
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
    assert_eq!(nomalize_whitespace(expected), nomalize_whitespace(&envrc))
}
