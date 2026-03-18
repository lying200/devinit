use tera::{Context, Tera};

use crate::schema::ProjectContext;

const DEVENV_TEMPLATE: &str = include_str!("../../templates/devenv.nix.tera");

pub fn render_devenv(project_ctx: &ProjectContext) -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("devenv", DEVENV_TEMPLATE)
        .expect("load template err");
    let mut tera_ctx = Context::new();
    tera_ctx.insert("project_ctx", project_ctx);
    tera.render("devenv", &tera_ctx)
        .expect("render template err")
}
