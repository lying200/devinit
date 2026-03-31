use std::io;

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
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    let target_dir = &cli.path;

    if !target_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("init target does not exist: {}", target_dir.display()),
        ));
    }

    if !cli.force {
        if let Some(found) = detect_existing_environment(target_dir)? {
            println!(
                "existing direnv/devenv/nix environment detected ({found}), skipping initialization"
            );
            println!("use --force to overwrite");
            return Ok(());
        }
    }

    let languages = resolve_languages_config(target_dir, &cli.lang, cli.yes)?;

    if languages.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no languages selected, nothing to generate",
        ));
    }

    let ctx = ProjectContext {
        languages,
        services: vec![], // TODO: services 检测/配置
        tools: vec![],    // TODO: tools 检测/配置
    };

    let output_files = plan_files(&ctx);
    write_files(target_dir, &output_files)?;

    if !cli.yes {
        handle_git_ignore(target_dir)?;
    }

    println!("\nGenerated files:");
    for file in &output_files {
        println!("  {}", file.filename);
    }
    println!("\ndevenv init success!");
    println!("Run \"direnv allow\" to activate the environment.");
    Ok(())
}

fn handle_git_ignore(target_dir: &std::path::Path) -> io::Result<()> {
    if find_git_repo_root(target_dir).is_none() {
        println!("git not initialized, skipping ignore handling");
        return Ok(());
    }

    let ignore_mode = prompt_ignore_mode();
    let outcome = apply_ignore_mode(target_dir, ignore_mode)?;

    if !outcome.tracked_files.is_empty() {
        println!("ignored patterns were added, but tracked files remain tracked by git");
        for path in outcome.tracked_files {
            println!("- {path}");
        }
    }
    Ok(())
}

fn resolve_languages_config(
    target_dir: &std::path::Path,
    cli_langs: &[LanguageChoice],
    non_interactive: bool,
) -> io::Result<Vec<Language>> {
    // --lang provided: use explicit languages
    if !cli_langs.is_empty() {
        if non_interactive {
            return Ok(cli_langs.iter().map(|c| c.to_default_language()).collect());
        }
        return Ok(cli_langs.iter().map(|&c| prompt_language_config(c)).collect());
    }

    // Run detection
    let detection = detect_project(target_dir)?;

    // --yes: accept all detected languages with default config
    if non_interactive {
        return match detection {
            DetectionOutcome::Matches { candidates } => {
                Ok(candidates.into_iter().map(|c| c.language).collect())
            }
            DetectionOutcome::NoMatch => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no languages detected and --yes specified, use --lang to specify languages",
            )),
        };
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

