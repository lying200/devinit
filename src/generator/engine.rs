use tera::{Context, Tera};

use crate::schema::ProjectContext;

const DEVENV_NIX_TEMPLATE: &str = include_str!("../../templates/devenv.nix.tera");
const DEVENV_YAML_TEMPLATE: &str = include_str!("../../templates/devenv.yaml.tera");
const ENVRC_TEMPLATE: &str = include_str!("../../templates/.envrc.tera");

fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Renders `devenv.nix` from the project context.
///
/// # Panics
///
/// Panics if the embedded template fails to load or render.
#[must_use]
pub fn render_devenv_nix(project_ctx: &ProjectContext) -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("devenv-nix", DEVENV_NIX_TEMPLATE)
        .expect("load template err");
    let mut tera_ctx = Context::new();
    tera_ctx.insert("project_ctx", project_ctx);
    normalize_line_endings(
        &tera.render("devenv-nix", &tera_ctx)
            .expect("render template err"),
    )
}

/// Renders `devenv.yaml` from the project context.
///
/// # Panics
///
/// Panics if the embedded template fails to load or render.
#[must_use]
pub fn render_devenv_yaml(project_ctx: &ProjectContext) -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("devenv-yaml", DEVENV_YAML_TEMPLATE)
        .expect("load template err");
    let mut tera_ctx = Context::new();
    tera_ctx.insert("project_ctx", project_ctx);
    normalize_line_endings(
        &tera.render("devenv-yaml", &tera_ctx)
            .expect("render template err"),
    )
}

/// Renders `.envrc` content.
///
/// # Panics
///
/// Panics if the embedded template fails to load or render.
#[must_use]
pub fn render_envrc() -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("envrc", ENVRC_TEMPLATE)
        .expect("load template err");
    let tera_ctx = Context::new();
    normalize_line_endings(
        &tera.render("envrc", &tera_ctx)
            .expect("render template err"),
    )
}
