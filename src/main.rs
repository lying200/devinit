use clap::Parser;
use devinit::{
    cli::{Cli, LanguageChoice},
    detection::{DetectionOutcome, detect_project},
    generator::{plan_files, write_files},
    git_ignore::{apply_ignore_mode, find_git_repo_root},
    init_guard::detect_existing_environment,
    prompt::{
        confirm_detected_config, prompt_ignore_mode, prompt_language_choice,
        prompt_language_config,
    },
    resolution::{ResolutionPlan, plan_language_resolution},
    schema::ProjectContext,
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

    let language = match resolve_language_config(&target_dir, cli.lang) {
        Ok(language) => language,
        Err(e) => {
            eprint!("resolve language config err: {e}");
            std::process::exit(1);
        }
    };

    let ctx = ProjectContext {
        language,
        services: vec![],
        tools: vec![],
    };

    let output_file = plan_files(&ctx);
    if let Err(e) = write_files(&target_dir, &output_file) {
        eprint!("generate devenv file err: {e}");
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

fn resolve_language_config(target_dir: &std::path::Path, cli_lang: Option<LanguageChoice>) -> std::io::Result<devinit::schema::Language> {
    let detection = if cli_lang.is_none() {
        detect_project(target_dir)?
    } else {
        DetectionOutcome::NoMatch
    };

    let use_detected = match &detection {
        DetectionOutcome::Match { candidate } => confirm_detected_config(candidate),
        DetectionOutcome::NoMatch => false,
    };

    let plan = plan_language_resolution(cli_lang, detection, use_detected);

    Ok(match plan {
        ResolutionPlan::Explicit(choice) => prompt_language_config(choice),
        ResolutionPlan::UseDetected(language) => language,
        ResolutionPlan::PromptManual => prompt_language_config(prompt_language_choice()),
    })
}
