use std::io;

use clap::Parser;
use devinit::{
    cli::{Cli, LanguageChoice, ServiceChoice},
    detection::{DetectionOutcome, detect_project, detect_services},
    generator::{plan_files, write_files},
    git_ignore::{apply_ignore_mode, find_git_repo_root},
    init_guard::detect_existing_environment,
    prompt::{
        confirm_detected_configs, confirm_detected_services, prompt_ignore_mode,
        prompt_language_choices, prompt_language_config, prompt_modify_detected,
        prompt_service_choices, prompt_service_config,
    },
    resolution::{ResolutionPlan, plan_language_resolution},
    schema::{Language, ProjectContext, Service},
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

    if !cli.force
        && let Some(found) = detect_existing_environment(target_dir)?
    {
        println!(
            "existing direnv/devenv/nix environment detected ({found}), skipping initialization"
        );
        println!("use --force to overwrite");
        return Ok(());
    }

    let languages = resolve_languages_config(target_dir, &cli.lang, cli.yes)?;

    if languages.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no languages selected, nothing to generate",
        ));
    }

    let services = resolve_services_config(target_dir, &cli.service, cli.yes)?;

    let ctx = ProjectContext {
        languages,
        services,
        tools: vec![], // TODO: tools 检测/配置
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
    if !ctx.services.is_empty() {
        println!("\nServices enabled:");
        for svc in &ctx.services {
            let name = match svc {
                Service::Postgres { .. } => "PostgreSQL",
                Service::Redis => "Redis",
                Service::Mysql { .. } => "MySQL",
            };
            println!("  {name}");
        }
        println!("Run \"devenv up\" to start services.");
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
    // --lang provided: use explicit languages (deduplicated)
    if !cli_langs.is_empty() {
        let mut deduped = cli_langs.to_vec();
        deduped.sort();
        deduped.dedup();
        if non_interactive {
            return Ok(deduped.iter().map(|c| c.to_default_language()).collect());
        }
        return Ok(deduped.iter().map(|&c| prompt_language_config(c)).collect());
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
        ResolutionPlan::UseDetected(languages) => prompt_modify_detected(languages),
        ResolutionPlan::PromptManual => {
            let choices = prompt_language_choices();
            choices
                .iter()
                .map(|&c| prompt_language_config(c))
                .collect()
        }
    })
}

fn resolve_services_config(
    target_dir: &std::path::Path,
    cli_services: &[ServiceChoice],
    non_interactive: bool,
) -> io::Result<Vec<Service>> {
    // --service provided: use explicit (deduplicated)
    if !cli_services.is_empty() {
        let mut deduped = cli_services.to_vec();
        deduped.sort();
        deduped.dedup();
        if non_interactive {
            return Ok(deduped.iter().map(|c| c.to_default_service()).collect());
        }
        return Ok(deduped.iter().map(|&c| prompt_service_config(c)).collect());
    }

    // Detect from docker-compose
    let candidates = detect_services(target_dir)?;

    if non_interactive {
        // --yes: auto-accept detected services, skip if none
        return Ok(candidates.into_iter().map(|c| c.service).collect());
    }

    // Interactive flow
    if candidates.is_empty() {
        // No compose file: prompt user to select
        let choices = prompt_service_choices();
        return Ok(choices
            .iter()
            .map(|&c| prompt_service_config(c))
            .collect());
    }

    // Compose file found: confirm detected services
    let confirmed = confirm_detected_services(&candidates);
    let services: Vec<Service> = confirmed
        .iter()
        .filter_map(|&i| candidates.get(i).map(|c| c.service.clone()))
        .collect();

    Ok(services)
}
