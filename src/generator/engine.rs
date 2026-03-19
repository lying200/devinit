use tera::{Context, Tera};

use crate::schema::ProjectContext;

const DEVENV_NIX_TEMPLATE: &str = include_str!("../../templates/devenv.nix.tera");
const DEVENV_YAML_TEMPLATE: &str = include_str!("../../templates/devenv.yaml.tera");
const ENVRC_TEMPLATE: &str = include_str!("../../templates/.envrc.tera");

pub fn render_devenv_nix(project_ctx: &ProjectContext) -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("devenv-nix", DEVENV_NIX_TEMPLATE)
        .expect("load template err");
    let mut tera_ctx = Context::new();
    tera_ctx.insert("project_ctx", project_ctx);
    tera.render("devenv-nix", &tera_ctx)
        .expect("render template err")
}

pub fn render_devenv_yaml(project_ctx: &ProjectContext) -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("devenv-yaml", DEVENV_YAML_TEMPLATE)
        .expect("load template err");
    let mut tera_ctx = Context::new();
    tera_ctx.insert("project_ctx", project_ctx);
    tera.render("devenv-yaml", &tera_ctx)
        .expect("render template err")
}

pub fn render_envrc() -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("envrc", ENVRC_TEMPLATE)
        .expect("load template err");
    let tera_ctx = Context::new();
    tera.render("envrc", &tera_ctx)
        .expect("render template err")
}
