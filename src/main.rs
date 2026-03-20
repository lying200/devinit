use clap::Parser;
use devinit::{
    cli::{Cli, LanguageChoice},
    generator::{plan_files, write_files},
    schema::{Language, ProjectContext},
};
use dialoguer::{Select, theme::ColorfulTheme};

fn main() {
    let cli = Cli::parse();
    let lang_choice = match cli.lang {
        Some(l) => l,
        None => {
            let options = vec!["Rust", "Python", "Go", "Java", "Nodejs"];
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select language")
                .default(0)
                .items(&options)
                .interact()
                .expect("select err");
            match selection {
                0 => LanguageChoice::Rust,
                1 => LanguageChoice::Python,
                2 => LanguageChoice::Go,
                3 => LanguageChoice::Java,
                4 => LanguageChoice::Nodejs,
                _ => unreachable!(),
            }
        }
    };

    let language = match lang_choice {
        LanguageChoice::Rust => Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: None,
        },
        LanguageChoice::Python => Language::Python,
        LanguageChoice::Go => Language::Go,
        LanguageChoice::Java => Language::Java,
        LanguageChoice::Nodejs => Language::Nodejs,
    };

    let ctx = ProjectContext {
        language,
        services: vec![],
        tools: vec![],
    };

    let output_file = plan_files(&ctx);
    let current_dir = cli.path;
    if let Err(e) = write_files(&current_dir, &output_file) {
        eprint!("generate devenv file err: {}", e);
        std::process::exit(1);
    }
    println!("devenv init success!");
}
