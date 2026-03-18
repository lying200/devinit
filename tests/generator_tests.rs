use devinit::generator::render_devenv;
use devinit::schema::{Language, ProjectContext, Service};

fn nomalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[test]
fn test_render_devenv() {
    let project_ctx = ProjectContext {
        project_name: "test_project".to_string(),
        project_path: "/path/to/test_project".to_string(),
        language: Language::Rust,
        services: vec![Service {
            name: "redis".to_string(),
            version: None,
        }],
        tools: vec!["git".to_string()],
    };
    let devenv_conf = render_devenv(&project_ctx);
    let expected = r#"
        { pkgs, ... }:

        {
        packages = [
            pkgs.git
        ];

        languages.rust = true;
        }
        "#;
    assert_eq!(
        nomalize_whitespace(expected),
        nomalize_whitespace(&devenv_conf)
    )
}
