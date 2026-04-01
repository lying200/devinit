use devinit::generator::{render_devenv_nix, render_devenv_yaml, render_envrc};
use devinit::schema::{Language, ProjectContext, Service};


#[test]
fn test_render_rust_base_without_tools_has_no_git_package() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: None,
        }],
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
        languages: vec![Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: None,
        }],
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
        languages: vec![Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: None,
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n  ];\n\n  languages.rust = {\n    enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_rust_with_channel() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Rust {
            channel: Some("stable".to_string()),
            version: None,
            components: None,
            targets: None,
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n  ];\n\n  languages.rust = {\n    enable = true;\n    channel = \"stable\";\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_rust_with_version() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Rust {
            channel: None,
            version: Some("1.81.0".to_string()),
            components: None,
            targets: None,
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n  ];\n\n  languages.rust = {\n    enable = true;\n    version = \"1.81.0\";\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_rust_with_components() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Rust {
            channel: None,
            version: None,
            components: Some(vec!["clippy".to_string()]),
            targets: None,
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n  ];\n\n  languages.rust = {\n    enable = true;\n    components = [\n      \"clippy\"\n    ];\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_rust_with_targets() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: Some(vec!["wasm32-unknown-unknown".to_string()]),
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n  ];\n\n  languages.rust = {\n    enable = true;\n    targets = [\n      \"wasm32-unknown-unknown\"\n    ];\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_devenv_yaml() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: None,
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = "inputs:\n  nixpkgs:\n    url: github:cachix/devenv-nixpkgs/rolling\n\n  rust-overlay:\n    url: github:oxalica/rust-overlay\n    inputs:\n      nixpkgs:\n        follows: nixpkgs\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_python_base() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Python {
            version: None,
            package: None,
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.python = {\n    enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_python_with_version() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Python {
            version: Some("3.11".to_string()),
            package: None,
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.python = {\n    enable = true;\n    version = \"3.11\";\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_python_with_package() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Python {
            version: None,
            package: Some("pkgs.python311".to_string()),
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.python = {\n    enable = true;\n    package = pkgs.python311;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_python_with_uv() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Python {
            version: None,
            package: None,
            uv_enable: Some(true),
            venv_enable: None,
            venv_quiet: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.python = {\n    enable = true;\n    uv.enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_python_with_venv() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Python {
            version: None,
            package: None,
            uv_enable: None,
            venv_enable: Some(true),
            venv_quiet: Some(true),
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.python = {\n    enable = true;\n    venv.enable = true;\n    venv.quiet = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_devenv_yaml_for_python_with_version() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Python {
            version: Some("3.11".to_string()),
            package: None,
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = "inputs:\n  nixpkgs:\n    url: github:cachix/devenv-nixpkgs/rolling\n\n  nixpkgs-python:\n    url: github:cachix/nixpkgs-python\n    inputs:\n      nixpkgs:\n        follows: nixpkgs\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_devenv_yaml_for_python_with_package_only() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Python {
            version: None,
            package: Some("pkgs.python311".to_string()),
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = "inputs:\n  nixpkgs:\n    url: github:cachix/devenv-nixpkgs/rolling\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_go_base() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Go {
            version: None,
            package: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.go = {\n    enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_go_with_version() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Go {
            version: Some("1.22.0".to_string()),
            package: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.go = {\n    enable = true;\n    version = \"1.22.0\";\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_go_with_package() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Go {
            version: None,
            package: Some("pkgs.go_1_24".to_string()),
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.go = {\n    enable = true;\n    package = pkgs.go_1_24;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_devenv_yaml_for_go_with_version() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Go {
            version: Some("1.22.0".to_string()),
            package: None,
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = "inputs:\n  nixpkgs:\n    url: github:cachix/devenv-nixpkgs/rolling\n\n  go-overlay:\n    url: github:purpleclay/go-overlay\n    inputs:\n      nixpkgs:\n        follows: nixpkgs\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_devenv_yaml_for_go_with_package_only() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Go {
            version: None,
            package: Some("pkgs.go_1_24".to_string()),
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = "inputs:\n  nixpkgs:\n    url: github:cachix/devenv-nixpkgs/rolling\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_java_base() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.java = {\n    enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_java_with_jdk_package() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Java {
            jdk_package: Some("pkgs.jdk17".to_string()),
            gradle_enable: None,
            maven_enable: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.java = {\n    enable = true;\n    jdk.package = pkgs.jdk17;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_java_with_gradle() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Java {
            jdk_package: None,
            gradle_enable: Some(true),
            maven_enable: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.java = {\n    enable = true;\n    gradle.enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_java_with_maven() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: Some(true),
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.java = {\n    enable = true;\n    maven.enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_devenv_yaml_for_java() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Java {
            jdk_package: Some("pkgs.jdk17".to_string()),
            gradle_enable: Some(true),
            maven_enable: Some(true),
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = "inputs:\n  nixpkgs:\n    url: github:cachix/devenv-nixpkgs/rolling\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_javascript_base() {
    let project_ctx = ProjectContext {
        languages: vec![Language::JavaScript {
            package: None,
            package_manager: None,
            corepack_enable: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.javascript = {\n    enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_javascript_with_package() {
    let project_ctx = ProjectContext {
        languages: vec![Language::JavaScript {
            package: Some("pkgs.nodejs_22".to_string()),
            package_manager: None,
            corepack_enable: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.javascript = {\n    enable = true;\n    package = pkgs.nodejs_22;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_javascript_with_npm() {
    let project_ctx = ProjectContext {
        languages: vec![Language::JavaScript {
            package: None,
            package_manager: Some("npm".to_string()),
            corepack_enable: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.javascript = {\n    enable = true;\n    npm.enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_javascript_with_pnpm() {
    let project_ctx = ProjectContext {
        languages: vec![Language::JavaScript {
            package: None,
            package_manager: Some("pnpm".to_string()),
            corepack_enable: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.javascript = {\n    enable = true;\n    pnpm.enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_javascript_with_yarn() {
    let project_ctx = ProjectContext {
        languages: vec![Language::JavaScript {
            package: None,
            package_manager: Some("yarn".to_string()),
            corepack_enable: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.javascript = {\n    enable = true;\n    yarn.enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_javascript_with_corepack() {
    let project_ctx = ProjectContext {
        languages: vec![Language::JavaScript {
            package: None,
            package_manager: None,
            corepack_enable: Some(true),
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.javascript = {\n    enable = true;\n    corepack.enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_javascript_with_bun() {
    let project_ctx = ProjectContext {
        languages: vec![Language::JavaScript {
            package: None,
            package_manager: Some("bun".to_string()),
            corepack_enable: None,
        }],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.javascript = {\n    enable = true;\n    bun.enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_devenv_yaml_for_javascript() {
    let project_ctx = ProjectContext {
        languages: vec![Language::JavaScript {
            package: Some("pkgs.nodejs_22".to_string()),
            package_manager: Some("pnpm".to_string()),
            corepack_enable: Some(true),
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = "inputs:\n  nixpkgs:\n    url: github:cachix/devenv-nixpkgs/rolling\n";
    assert_eq!(expected, devenv_conf);
}

// Multi-language tests

#[test]
fn test_render_multi_language_go_and_javascript() {
    let project_ctx = ProjectContext {
        languages: vec![
            Language::Go {
                version: Some("1.22.0".to_string()),
                package: None,
            },
            Language::JavaScript {
                package: None,
                package_manager: Some("pnpm".to_string()),
                corepack_enable: None,
            },
        ],
        services: vec![],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    let expected = "{ pkgs, ... }:\n\n{\n  packages = [\n    pkgs.git\n  ];\n\n  languages.go = {\n    enable = true;\n    version = \"1.22.0\";\n  };\n\n  languages.javascript = {\n    enable = true;\n    pnpm.enable = true;\n  };\n}\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_devenv_yaml_multi_language_go_version_and_javascript() {
    let project_ctx = ProjectContext {
        languages: vec![
            Language::Go {
                version: Some("1.22.0".to_string()),
                package: None,
            },
            Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
        ],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    let expected = "inputs:\n  nixpkgs:\n    url: github:cachix/devenv-nixpkgs/rolling\n\n  go-overlay:\n    url: github:purpleclay/go-overlay\n    inputs:\n      nixpkgs:\n        follows: nixpkgs\n";
    assert_eq!(expected, devenv_conf);
}

#[test]
fn test_render_multi_language_rust_and_javascript() {
    let project_ctx = ProjectContext {
        languages: vec![
            Language::Rust {
                channel: Some("stable".to_string()),
                version: None,
                components: None,
                targets: None,
            },
            Language::JavaScript {
                package: None,
                package_manager: Some("npm".to_string()),
                corepack_enable: None,
            },
        ],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    assert!(devenv_conf.contains("languages.rust"));
    assert!(devenv_conf.contains("languages.javascript"));
    assert!(devenv_conf.contains("channel = \"stable\""));
    assert!(devenv_conf.contains("npm.enable = true"));
}

#[test]
fn test_render_devenv_yaml_multi_language_rust_and_python_with_version() {
    let project_ctx = ProjectContext {
        languages: vec![
            Language::Rust {
                channel: None,
                version: None,
                components: None,
                targets: None,
            },
            Language::Python {
                version: Some("3.11".to_string()),
                package: None,
                uv_enable: None,
                venv_enable: None,
                venv_quiet: None,
            },
        ],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_yaml(&project_ctx);
    assert!(devenv_conf.contains("rust-overlay"));
    assert!(devenv_conf.contains("nixpkgs-python"));
}

#[test]
fn test_render_envrc() {
    let envrc = render_envrc();
    let expected = "# devinit: start\neval \"$(devenv direnvrc)\"\n\n# You can pass flags to the devenv command\n# For example: use devenv --impure --option services.postgres.enable:bool true\nuse devenv\n# devinit: end\n";
    assert_eq!(expected, envrc);
}

#[test]
fn test_render_postgres_service() {
    let project_ctx = ProjectContext {
        languages: vec![],
        services: vec![Service::Postgres { package: None }],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    assert!(devenv_conf.contains("services.postgres"));
    assert!(devenv_conf.contains("enable = true;"));
    assert!(devenv_conf.contains("listen_addresses = \"127.0.0.1\";"));
}

#[test]
fn test_render_postgres_with_package() {
    let project_ctx = ProjectContext {
        languages: vec![],
        services: vec![Service::Postgres {
            package: Some("pkgs.postgresql_16".to_string()),
        }],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    assert!(devenv_conf.contains("services.postgres"));
    assert!(devenv_conf.contains("package = pkgs.postgresql_16;"));
}

#[test]
fn test_render_redis_service() {
    let project_ctx = ProjectContext {
        languages: vec![],
        services: vec![Service::Redis],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    assert!(devenv_conf.contains("services.redis.enable = true;"));
}

#[test]
fn test_render_mysql_service() {
    let project_ctx = ProjectContext {
        languages: vec![],
        services: vec![Service::Mysql { package: None }],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    assert!(devenv_conf.contains("services.mysql ="));
    assert!(devenv_conf.contains("enable = true;"));
    // Should NOT contain postgres
    assert!(!devenv_conf.contains("services.postgres"));
}

#[test]
fn test_render_mysql_with_package() {
    let project_ctx = ProjectContext {
        languages: vec![],
        services: vec![Service::Mysql {
            package: Some("pkgs.mysql80".to_string()),
        }],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    assert!(devenv_conf.contains("services.mysql ="));
    assert!(devenv_conf.contains("package = pkgs.mysql80;"));
}

#[test]
fn test_render_multi_services() {
    let project_ctx = ProjectContext {
        languages: vec![],
        services: vec![
            Service::Postgres { package: None },
            Service::Redis,
        ],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    assert!(devenv_conf.contains("services.postgres"));
    assert!(devenv_conf.contains("services.redis.enable = true;"));
}

#[test]
fn test_render_no_services() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Go {
            version: None,
            package: None,
        }],
        services: vec![],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    assert!(!devenv_conf.contains("services."));
}

#[test]
fn test_render_language_and_service_together() {
    let project_ctx = ProjectContext {
        languages: vec![Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: None,
        }],
        services: vec![Service::Postgres { package: None }, Service::Redis],
        tools: vec![],
    };
    let devenv_conf = render_devenv_nix(&project_ctx);
    assert!(devenv_conf.contains("languages.java"));
    assert!(devenv_conf.contains("services.postgres"));
    assert!(devenv_conf.contains("services.redis.enable = true;"));
}
