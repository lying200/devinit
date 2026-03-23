use devinit::{generator::plan_files, schema::ProjectContext};

#[test]
fn test_plan_files_for_rust_project() {
    let ctx = ProjectContext {
        language: devinit::schema::Language::Rust {
            channel: Some("stable".to_string()),
            version: Some("1.81.0".to_string()),
            components: Some(vec!["rustfmt".to_string()]),
            targets: Some(vec!["x86_64-unknown-linux-gnu".to_string()]),
        },
        services: Vec::new(),
        tools: vec!["git".to_string()],
    };

    let generated_files = plan_files(&ctx);

    assert_eq!(
        generated_files.len(),
        3,
        "Rust 项目应该生成 3 个基础配置文件"
    );

    let filenames: Vec<&str> = generated_files
        .iter()
        .map(|f| f.filename.as_str())
        .collect();

    assert!(filenames.contains(&"devenv.nix"), "缺少 devenv.nix");
    assert!(filenames.contains(&"devenv.yaml"), "缺少 devenv.yaml");
    assert!(filenames.contains(&".envrc"), "缺少 .envrc");

    let nix_file = generated_files
        .iter()
        .find(|f| f.filename == "devenv.nix")
        .unwrap();
    assert!(
        nix_file.content.contains("enable = true"),
        "Nix 文件中应该开启 Rust"
    );
    assert!(
        nix_file.content.contains("\"rustfmt\""),
        "Nix 文件中应该包含自定义的 rustfmt 组件"
    );
}

#[test]
fn test_plan_files_for_javascript_project() {
    let ctx = ProjectContext {
        language: devinit::schema::Language::JavaScript {
            package: Some("pkgs.nodejs_22".to_string()),
            package_manager: Some("pnpm".to_string()),
            corepack_enable: Some(true),
        },
        services: Vec::new(),
        tools: vec!["git".to_string()],
    };

    let generated_files = plan_files(&ctx);

    assert_eq!(
        generated_files.len(),
        3,
        "JavaScript 项目应该生成 3 个基础配置文件"
    );

    let filenames: Vec<&str> = generated_files
        .iter()
        .map(|f| f.filename.as_str())
        .collect();

    assert!(filenames.contains(&"devenv.nix"), "缺少 devenv.nix");
    assert!(filenames.contains(&"devenv.yaml"), "缺少 devenv.yaml");
    assert!(filenames.contains(&".envrc"), "缺少 .envrc");

    let nix_file = generated_files
        .iter()
        .find(|f| f.filename == "devenv.nix")
        .unwrap();
    assert!(
        nix_file.content.contains("languages.javascript"),
        "Nix 文件中应该包含 JavaScript 语言配置"
    );
}

#[test]
fn test_plan_files_for_java_project() {
    let ctx = ProjectContext {
        language: devinit::schema::Language::Java {
            jdk_package: Some("pkgs.jdk17".to_string()),
            gradle_enable: Some(true),
            maven_enable: Some(true),
        },
        services: Vec::new(),
        tools: vec!["git".to_string()],
    };

    let generated_files = plan_files(&ctx);

    assert_eq!(
        generated_files.len(),
        3,
        "Java 项目应该生成 3 个基础配置文件"
    );

    let filenames: Vec<&str> = generated_files
        .iter()
        .map(|f| f.filename.as_str())
        .collect();

    assert!(filenames.contains(&"devenv.nix"), "缺少 devenv.nix");
    assert!(filenames.contains(&"devenv.yaml"), "缺少 devenv.yaml");
    assert!(filenames.contains(&".envrc"), "缺少 .envrc");

    let nix_file = generated_files
        .iter()
        .find(|f| f.filename == "devenv.nix")
        .unwrap();
    assert!(
        nix_file.content.contains("languages.java"),
        "Nix 文件中应该包含 Java 语言配置"
    );
}

#[test]
fn test_plan_files_for_go_project() {
    let ctx = ProjectContext {
        language: devinit::schema::Language::Go {
            version: Some("1.22.0".to_string()),
            package: Some("pkgs.go_1_24".to_string()),
        },
        services: Vec::new(),
        tools: vec!["git".to_string()],
    };

    let generated_files = plan_files(&ctx);

    assert_eq!(generated_files.len(), 3, "Go 项目应该生成 3 个基础配置文件");

    let filenames: Vec<&str> = generated_files
        .iter()
        .map(|f| f.filename.as_str())
        .collect();

    assert!(filenames.contains(&"devenv.nix"), "缺少 devenv.nix");
    assert!(filenames.contains(&"devenv.yaml"), "缺少 devenv.yaml");
    assert!(filenames.contains(&".envrc"), "缺少 .envrc");

    let nix_file = generated_files
        .iter()
        .find(|f| f.filename == "devenv.nix")
        .unwrap();
    assert!(
        nix_file.content.contains("languages.go"),
        "Nix 文件中应该包含 Go 语言配置"
    );
}

#[test]
fn test_plan_files_for_python_project() {
    let ctx = ProjectContext {
        language: devinit::schema::Language::Python {
            version: Some("3.11".to_string()),
            package: Some("pkgs.python311".to_string()),
            uv_enable: Some(true),
            venv_enable: Some(true),
            venv_quiet: Some(true),
        },
        services: Vec::new(),
        tools: vec!["git".to_string()],
    };

    let generated_files = plan_files(&ctx);

    assert_eq!(
        generated_files.len(),
        3,
        "Python 项目应该生成 3 个基础配置文件"
    );

    let filenames: Vec<&str> = generated_files
        .iter()
        .map(|f| f.filename.as_str())
        .collect();

    assert!(filenames.contains(&"devenv.nix"), "缺少 devenv.nix");
    assert!(filenames.contains(&"devenv.yaml"), "缺少 devenv.yaml");
    assert!(filenames.contains(&".envrc"), "缺少 .envrc");

    let nix_file = generated_files
        .iter()
        .find(|f| f.filename == "devenv.nix")
        .unwrap();
    assert!(
        nix_file.content.contains("languages.python"),
        "Nix 文件中应该包含 Python 语言配置"
    );
}
