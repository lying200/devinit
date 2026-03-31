use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub lang: Vec<LanguageChoice>,

    /// Non-interactive mode: accept detected config and use defaults
    #[arg(short, long)]
    pub yes: bool,

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
