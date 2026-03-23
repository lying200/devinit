use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub lang: Option<LanguageChoice>,

    #[arg(default_value = ".")]
    pub path: PathBuf
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
