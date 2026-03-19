pub mod engine;

use std::path::Path;

pub use engine::*;

use crate::schema::ProjectContext;

#[derive(Debug, PartialEq, Eq)]
pub struct OutputFile {
    pub filename: String,
    pub content: String,
}

pub fn plan_files(ctx: &ProjectContext) -> Vec<OutputFile> {
    vec![
        OutputFile {
            filename: "devenv.nix".to_string(),
            content: engine::render_devenv_nix(ctx),
        },
        OutputFile {
            filename: "devenv.yaml".to_string(),
            content: engine::render_devenv_yaml(ctx),
        },
        OutputFile {
            filename: ".envrc".to_string(),
            content: engine::render_envrc(),
        },
    ]
}

pub fn write_files(target_dir: &Path, files: &[OutputFile]) -> Result<(), std::io::Error> {
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)?;
    }

    for file in files {
        let path = target_dir.join(&file.filename);
        std::fs::write(&path, &file.content)?;
    }
    Ok(())
}
