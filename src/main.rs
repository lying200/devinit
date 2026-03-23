use clap::Parser;
use devinit::{
    cli::{Cli, LanguageChoice},
    generator::{plan_files, write_files},
    git_ignore::{apply_ignore_mode, find_git_repo_root},
    init_guard::{detect_existing_environment, initialize_git_repository, target_dir_was_empty},
    prompt::{
        prompt_go_config, prompt_ignore_mode, prompt_java_config, prompt_javascript_config,
        prompt_python_config, prompt_rust_config,
    },
    schema::ProjectContext,
};
use dialoguer::{Select, theme::ColorfulTheme};

fn main() {
    let cli = Cli::parse();
    let target_dir = cli.path;
    let existed_before = target_dir.exists();

    if let Err(e) = std::fs::create_dir_all(&target_dir) {
        eprint!("create init target err: {}", e);
        std::process::exit(1);
    }

    match detect_existing_environment(&target_dir) {
        Ok(Some(found)) => {
            println!(
                "existing direnv/devenv/nix environment detected ({}), skipping devinit initialization",
                found
            );
            return;
        }
        Ok(None) => {}
        Err(e) => {
            eprint!("inspect init target err: {}", e);
            std::process::exit(1);
        }
    }

    match target_dir_was_empty(&target_dir, existed_before) {
        Ok(true) => {
            if let Err(e) = initialize_git_repository(&target_dir) {
                eprint!("git init err: {}", e);
                std::process::exit(1);
            }
        }
        Ok(false) => {}
        Err(e) => {
            eprint!("inspect init target err: {}", e);
            std::process::exit(1);
        }
    }

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
    if let Err(e) = write_files(&target_dir, &output_file) {
        eprint!("generate devenv file err: {}", e);
        std::process::exit(1);
    }

    if find_git_repo_root(&target_dir).is_none() {
        println!("git not initialized, skipping ignore handling");
    } else {
        let ignore_mode = prompt_ignore_mode();
        match apply_ignore_mode(&target_dir, ignore_mode) {
            Ok(outcome) => {
                if !outcome.tracked_files.is_empty() {
                    println!(
                        "ignored patterns were added, but tracked files remain tracked by git"
                    );
                    for path in outcome.tracked_files {
                        println!("- {}", path);
                    }
                }
            }
            Err(e) => {
                eprint!("apply git ignore err: {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("devenv init success!");
    println!("use \"direnv allow\" to activate the environment.")
}
