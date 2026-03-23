use clap::Parser;
use devinit::{
    cli::{Cli, LanguageChoice},
    generator::{plan_files, write_files},
    prompt::{
        prompt_go_config,
        prompt_java_config,
        prompt_javascript_config,
        prompt_python_config,
        prompt_rust_config,
    },
    schema::ProjectContext,
};
use dialoguer::{Select, theme::ColorfulTheme};

fn main() {
    let cli = Cli::parse();
    let lang_choice = match cli.lang {
        Some(l) => l,
        None => {
            let options = vec!["Rust", "Python", "Go", "Java", "JavaScript"];
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
                4 => LanguageChoice::JavaScript,
                _ => unreachable!(),
            }
        }
    };

    let language = match lang_choice {
        LanguageChoice::Rust => prompt_rust_config(),
        LanguageChoice::Python => prompt_python_config(),
        LanguageChoice::Go => prompt_go_config(),
        LanguageChoice::Java => prompt_java_config(),
        LanguageChoice::JavaScript => prompt_javascript_config(),
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
    println!("use \"direnv allow\" to activate the environment.")
}
