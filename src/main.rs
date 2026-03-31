use clap::Parser;
use devinit::{
    cli::{Cli, LanguageChoice},
    detection::{DetectionOutcome, detect_project},
    generator::{plan_files, write_files},
    git_ignore::{apply_ignore_mode, find_git_repo_root},
    init_guard::detect_existing_environment,
    prompt::{
        confirm_detected_configs, prompt_ignore_mode, prompt_language_choices,
        prompt_language_config,
    },
    resolution::{ResolutionPlan, plan_language_resolution},
    schema::{Language, ProjectContext},
};

fn main() {
    let cli = Cli::parse();
    let target_dir = cli.path;

    if !target_dir.exists() {
        eprint!(
            "init target does not exist: {path}",
            path = target_dir.display()
        );
        std::process::exit(1);
    }

    match detect_existing_environment(&target_dir) {
        Ok(Some(found)) => {
            println!(
                "existing direnv/devenv/nix environment detected ({found}), skipping devinit initialization"
            );
            return;
        }
        Ok(None) => {}
        Err(e) => {
            eprint!("inspect init target err: {e}");
            std::process::exit(1);
        }
    }

    let languages = match resolve_languages_config(&target_dir, &cli.lang, cli.yes) {
        Ok(languages) => languages,
        Err(e) => {
            eprint!("resolve language config err: {e}");
            std::process::exit(1);
        }
    };

    if languages.is_empty() {
        eprintln!("no languages selected, nothing to generate");
        std::process::exit(1);
    }

    let ctx = ProjectContext {
        languages,
        services: vec![], // TODO: services 检测/配置
        tools: vec![],    // TODO: tools 检测/配置
    };

    let output_file = plan_files(&ctx);
    if let Err(e) = write_files(&target_dir, &output_file) {
        eprint!("generate devenv file err: {e}");
        std::process::exit(1);
    }

    if cli.yes {
        println!("devenv init success!");
        println!("use \"direnv allow\" to activate the environment.");
        return;
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
                        println!("- {path}");
                    }
                }
            }
            Err(e) => {
                eprint!("apply git ignore err: {e}");
                std::process::exit(1);
            }
        }
    }

    println!("devenv init success!");
    println!("use \"direnv allow\" to activate the environment.")
}

fn resolve_languages_config(
    target_dir: &std::path::Path,
    cli_langs: &[LanguageChoice],
    non_interactive: bool,
) -> std::io::Result<Vec<Language>> {
    // --lang provided: use explicit languages
    if !cli_langs.is_empty() {
        if non_interactive {
            return Ok(cli_langs.iter().map(|&c| default_language(c)).collect());
        }
        return Ok(cli_langs.iter().map(|&c| prompt_language_config(c)).collect());
    }

    // Run detection
    let detection = detect_project(target_dir)?;

    // --yes: accept all detected languages with default config
    if non_interactive {
        return Ok(match detection {
            DetectionOutcome::Matches { candidates } => {
                candidates.into_iter().map(|c| c.language).collect()
            }
            DetectionOutcome::NoMatch => {
                eprintln!("no languages detected and --yes specified, cannot proceed without --lang");
                std::process::exit(1);
            }
        });
    }

    // Interactive flow
    let confirmed_indices = match &detection {
        DetectionOutcome::Matches { candidates } => confirm_detected_configs(candidates),
        DetectionOutcome::NoMatch => vec![],
    };

    let plan = plan_language_resolution(cli_langs, detection, &confirmed_indices);

    Ok(match plan {
        ResolutionPlan::Explicit(choices) => choices
            .iter()
            .map(|&c| prompt_language_config(c))
            .collect(),
        ResolutionPlan::UseDetected(languages) => languages,
        ResolutionPlan::PromptManual => {
            let choices = prompt_language_choices();
            choices
                .iter()
                .map(|&c| prompt_language_config(c))
                .collect()
        }
    })
}

fn default_language(choice: LanguageChoice) -> Language {
    match choice {
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
