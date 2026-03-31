pub mod engine;

use std::path::Path;

pub use engine::*;

use crate::schema::ProjectContext;

#[derive(Debug, PartialEq, Eq)]
pub struct OutputFile {
    pub filename: String,
    pub content: String,
}

#[must_use]
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

/// Writes generated files to the target directory.
///
/// # Errors
///
/// Returns any I/O error from creating directories or writing files.
pub fn write_files(target_dir: &Path, files: &[OutputFile]) -> Result<(), std::io::Error> {
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)?;
    }

    for file in files {
        let path = target_dir.join(&file.filename);
        if file.filename == ".envrc" {
            write_envrc(&path, &file.content)?;
        } else {
            std::fs::write(&path, &file.content)?;
        }
    }
    Ok(())
}

/// .envrc 使用追加模式：如果文件已存在且包含 devenv 配置则跳过，
/// 已存在但无 devenv 配置则追加，不存在则创建。
fn write_envrc(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if path.exists() {
        let existing = std::fs::read_to_string(path)?;
        if existing.contains("use devenv") {
            return Ok(());
        }
        let mut combined = existing;
        if !combined.ends_with('\n') {
            combined.push('\n');
        }
        combined.push('\n');
        combined.push_str(content);
        std::fs::write(path, combined)?;
    } else {
        std::fs::write(path, content)?;
    }
    Ok(())
}
