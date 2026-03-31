use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Initialize devenv development environment for your project",
    long_about = "Automatically detect project languages and generate devenv.nix, devenv.yaml, and .envrc configuration files."
)]
pub struct Cli {
    #[arg(short, long, value_delimiter = ',')]
    pub lang: Vec<LanguageChoice>,

    /// Non-interactive mode: accept detected config and use defaults
    #[arg(short, long)]
    pub yes: bool,

    /// Overwrite existing devenv/direnv/nix configuration
    #[arg(short, long)]
    pub force: bool,

    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum LanguageChoice {
    Rust,
    Python,
    Go,
    Java,
    #[value(name = "javascript")]
    JavaScript,
}

impl LanguageChoice {
    pub fn to_default_language(self) -> crate::schema::Language {
        use crate::schema::Language;
        match self {
            LanguageChoice::Rust => Language::Rust {
                channel: None,
                version: None,
                components: None,
                targets: None,
            },
            LanguageChoice::Python => Language::Python {
                version: None,
                package: None,
                uv_enable: None,
                venv_enable: None,
                venv_quiet: None,
            },
            LanguageChoice::Go => Language::Go {
                version: None,
                package: None,
            },
            LanguageChoice::Java => Language::Java {
                jdk_package: None,
                gradle_enable: None,
                maven_enable: None,
            },
            LanguageChoice::JavaScript => Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
        }
    }
}
