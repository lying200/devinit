use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};

use crate::cli::LanguageChoice;
use crate::detection::LanguageCandidate;
use crate::git_ignore::IgnoreMode;
use crate::schema::Language;

pub fn ignore_mode_from_selection(selection: usize) -> IgnoreMode {
    match selection {
        0 => IgnoreMode::None,
        1 => IgnoreMode::GitIgnore,
        2 => IgnoreMode::LocalExclude,
        _ => unreachable!(),
    }
}

pub fn prompt_ignore_mode() -> IgnoreMode {
    let options = vec![
        "Do nothing",
        "Add to .gitignore",
        "Add to local git exclude (.git/info/exclude, ignore devenv mechanism locally)",
    ];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("How to handle git ignore for devenv files?")
        .default(0)
        .items(&options)
        .interact()
        .expect("select err");

    ignore_mode_from_selection(selection)
}

pub fn prompt_language_choices() -> Vec<LanguageChoice> {
    let options = vec!["Rust", "Python", "Go", "Java", "JavaScript"];
    loop {
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select language(s)")
            .items(&options)
            .interact()
            .expect("select err");

        if selections.is_empty() {
            println!("please select at least one language");
            continue;
        }

        return selections
            .iter()
            .map(|&i| match i {
                0 => LanguageChoice::Rust,
                1 => LanguageChoice::Python,
                2 => LanguageChoice::Go,
                3 => LanguageChoice::Java,
                4 => LanguageChoice::JavaScript,
                _ => unreachable!(),
            })
            .collect();
    }
}

pub fn prompt_language_config(choice: LanguageChoice) -> Language {
    match choice {
        LanguageChoice::Rust => prompt_rust_config(),
        LanguageChoice::Python => prompt_python_config(),
        LanguageChoice::Go => prompt_go_config(),
        LanguageChoice::Java => prompt_java_config(),
        LanguageChoice::JavaScript => prompt_javascript_config(),
    }
}

pub fn format_detected_summary(candidate: &LanguageCandidate) -> String {
    let mut lines = vec![format!(
        "detected language: {}",
        detected_language_name(&candidate.language)
    )];

    if let Some(field_line) = detected_primary_field(candidate) {
        lines.push(field_line);
    }

    for reason in &candidate.reasons {
        lines.push(format!("- {reason}"));
    }

    lines.join("\n")
}

pub fn confirm_detected_configs(candidates: &[LanguageCandidate]) -> Vec<usize> {
    let labels: Vec<String> = candidates
        .iter()
        .map(|c| {
            let name = detected_language_name(&c.language);
            let detail = detected_primary_field(c).unwrap_or_default();
            if detail.is_empty() {
                name.to_string()
            } else {
                format!("{name} ({detail})")
            }
        })
        .collect();
    let defaults: Vec<bool> = vec![true; candidates.len()];

    MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Detected languages - select which to use")
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .expect("interact err")
}

fn detected_language_name(language: &Language) -> &'static str {
    match language {
        Language::Rust { .. } => "Rust",
        Language::Python { .. } => "Python",
        Language::Go { .. } => "Go",
        Language::Java { .. } => "Java",
        Language::JavaScript { .. } => "JavaScript",
    }
}

fn detected_primary_field(candidate: &LanguageCandidate) -> Option<String> {
    match &candidate.language {
        Language::Rust {
            channel: Some(channel),
            ..
        } => Some(format!("detected channel: {channel}")),
        Language::Rust {
            version: Some(version),
            ..
        } => Some(format!("detected version: {version}")),
        Language::Python {
            version: Some(version),
            ..
        } => Some(format!("detected version: {version}")),
        Language::Go {
            version: Some(version),
            ..
        } => Some(format!("detected version: {version}")),
        Language::Java {
            gradle_enable: Some(true),
            ..
        } => Some("detected build tool: gradle".to_string()),
        Language::Java {
            maven_enable: Some(true),
            ..
        } => Some("detected build tool: maven".to_string()),
        Language::JavaScript {
            package_manager: Some(manager),
            ..
        } => Some(format!("detected package manager: {manager}")),
        _ => None,
    }
}

pub fn prompt_rust_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("use default rust config?")
        .default(true)
        .interact()
        .expect("interact err exit");
    if use_default {
        return Language::Rust {
            channel: None,
            version: None,
            components: None,
            targets: None,
        };
    }

    let channels = vec!["nixpkgs", "stable", "beta", "nightly"];
    let channel_idx = Select::with_theme(&theme)
        .with_prompt("channel")
        .default(0)
        .items(&channels)
        .interact()
        .unwrap();
    let channel = channels[channel_idx].to_string();

    let version_input: String = Input::with_theme(&theme)
        .with_prompt("version")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    let version = if version_input.is_empty() {
        None
    } else {
        Some(version_input.trim().to_string())
    };

    let available_components = vec![
        "rustc",
        "cargo",
        "clippy",
        "rustfmt",
        "rust-analyzer",
        "rust-src",
    ];
    let default_selections = vec![true, true, true, true, true, false];

    let component_selections = MultiSelect::with_theme(&theme)
        .with_prompt("components")
        .items(&available_components)
        .defaults(&default_selections)
        .interact()
        .unwrap();
    let components = if component_selections.is_empty() {
        None
    } else {
        Some(
            component_selections
                .into_iter()
                .map(|i| available_components[i].to_string())
                .collect::<Vec<String>>(),
        )
    };

    let targets_input: String = Input::with_theme(&theme)
        .with_prompt("targets")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    let targets = if targets_input.is_empty() {
        None
    } else {
        Some(
            targets_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        )
    };

    Language::Rust {
        channel: Some(channel),
        version,
        components,
        targets,
    }
}

pub fn prompt_python_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("use default python config?")
        .default(true)
        .interact()
        .expect("interact err exit");
    if use_default {
        return Language::Python {
            version: None,
            package: None,
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        };
    }

    let version_input: String = Input::with_theme(&theme)
        .with_prompt("version")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    let version = if version_input.is_empty() {
        None
    } else {
        Some(version_input.trim().to_string())
    };

    let package_input: String = Input::with_theme(&theme)
        .with_prompt("package")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    let package = if package_input.is_empty() {
        None
    } else {
        Some(package_input.trim().to_string())
    };

    let uv_enable = Some(
        Confirm::with_theme(&theme)
            .with_prompt("enable uv?")
            .default(false)
            .interact()
            .expect("interact err exit"),
    );

    let venv_enabled = Confirm::with_theme(&theme)
        .with_prompt("enable venv?")
        .default(false)
        .interact()
        .expect("interact err exit");
    let venv_enable = Some(venv_enabled);

    let venv_quiet = if venv_enabled {
        Some(
            Confirm::with_theme(&theme)
                .with_prompt("quiet?")
                .default(false)
                .interact()
                .expect("interact err exit"),
        )
    } else {
        None
    };

    Language::Python {
        version,
        package,
        uv_enable,
        venv_enable,
        venv_quiet,
    }
}

pub fn prompt_go_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("use default go config?")
        .default(true)
        .interact()
        .expect("interact err exit");
    if use_default {
        return Language::Go {
            version: None,
            package: None,
        };
    }

    let version_input: String = Input::with_theme(&theme)
        .with_prompt("version")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    let version = if version_input.is_empty() {
        None
    } else {
        Some(version_input.trim().to_string())
    };

    let package_input: String = Input::with_theme(&theme)
        .with_prompt("package")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    let package = if package_input.is_empty() {
        None
    } else {
        Some(package_input.trim().to_string())
    };

    Language::Go { version, package }
}

pub fn prompt_java_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("use default java config?")
        .default(true)
        .interact()
        .expect("interact err exit");
    if use_default {
        return Language::Java {
            jdk_package: None,
            gradle_enable: None,
            maven_enable: None,
        };
    }

    let jdk_package_input: String = Input::with_theme(&theme)
        .with_prompt("jdk package")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    let jdk_package = if jdk_package_input.is_empty() {
        None
    } else {
        Some(jdk_package_input.trim().to_string())
    };

    let gradle_enable = Some(
        Confirm::with_theme(&theme)
            .with_prompt("enable gradle?")
            .default(false)
            .interact()
            .expect("interact err exit"),
    );

    let maven_enable = Some(
        Confirm::with_theme(&theme)
            .with_prompt("enable maven?")
            .default(false)
            .interact()
            .expect("interact err exit"),
    );

    Language::Java {
        jdk_package,
        gradle_enable,
        maven_enable,
    }
}

pub fn prompt_javascript_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("use default javascript config?")
        .default(true)
        .interact()
        .expect("interact err exit");
    if use_default {
        return Language::JavaScript {
            package: None,
            package_manager: None,
            corepack_enable: None,
        };
    }

    let package_input: String = Input::with_theme(&theme)
        .with_prompt("package")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    let package = if package_input.is_empty() {
        None
    } else {
        Some(package_input.trim().to_string())
    };

    let package_managers = vec!["none", "npm", "pnpm", "yarn", "bun"];
    let package_manager_idx = Select::with_theme(&theme)
        .with_prompt("package manager")
        .default(0)
        .items(&package_managers)
        .interact()
        .expect("interact err exit");
    let package_manager = if package_manager_idx == 0 {
        None
    } else {
        Some(package_managers[package_manager_idx].to_string())
    };

    let corepack_enable = Some(
        Confirm::with_theme(&theme)
            .with_prompt("enable corepack?")
            .default(false)
            .interact()
            .expect("interact err exit"),
    );

    Language::JavaScript {
        package,
        package_manager,
        corepack_enable,
    }
}
