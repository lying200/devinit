use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};

use crate::cli::{LanguageChoice, ServiceChoice};
use crate::detection::{LanguageCandidate, ServiceCandidate};
use crate::git_ignore::IgnoreMode;
use crate::schema::{Language, Service};
use crate::version_fetch;

#[must_use]
pub fn ignore_mode_from_selection(selection: usize) -> IgnoreMode {
    match selection {
        0 => IgnoreMode::None,
        1 => IgnoreMode::GitIgnore,
        2 => IgnoreMode::LocalExclude,
        _ => unreachable!(),
    }
}

/// Prompts the user to select a git ignore mode.
///
/// # Panics
///
/// Panics if the terminal interaction fails.
#[must_use]
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

/// Prompts the user to select one or more languages.
///
/// # Panics
///
/// Panics if the terminal interaction fails.
#[must_use]
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

#[must_use]
pub fn prompt_language_config(choice: LanguageChoice) -> Language {
    match choice {
        LanguageChoice::Rust => prompt_rust_config(),
        LanguageChoice::Python => prompt_python_config(),
        LanguageChoice::Go => prompt_go_config(),
        LanguageChoice::Java => prompt_java_config(),
        LanguageChoice::JavaScript => prompt_javascript_config(),
    }
}

#[must_use]
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

/// Displays detected languages and prompts the user to confirm selections.
///
/// # Panics
///
/// Panics if the terminal interaction fails.
#[must_use]
pub fn confirm_detected_configs(candidates: &[LanguageCandidate]) -> Vec<usize> {
    println!("\nProject analysis:");
    for candidate in candidates {
        let name = detected_language_name(&candidate.language);
        let detail = detected_primary_field(candidate);
        if let Some(d) = &detail {
            println!("  {name} - {d}");
        } else {
            println!("  {name}");
        }
        for reason in &candidate.reasons {
            println!("    - {reason}");
        }
    }
    println!();

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
        .with_prompt("Use detected languages?")
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .expect("interact err")
}

/// Prompts the user to optionally modify each detected language config.
///
/// # Panics
///
/// Panics if the terminal interaction fails.
#[must_use]
pub fn prompt_modify_detected(languages: Vec<Language>) -> Vec<Language> {
    let theme = ColorfulTheme::default();
    languages
        .into_iter()
        .map(|lang| {
            let name = detected_language_name(&lang);
            let modify = Confirm::with_theme(&theme)
                .with_prompt(format!("Modify {name} config?"))
                .default(false)
                .interact()
                .expect("interact err");
            if modify {
                prompt_language_config(language_to_choice(&lang))
            } else {
                lang
            }
        })
        .collect()
}

fn language_to_choice(language: &Language) -> LanguageChoice {
    match language {
        Language::Rust { .. } => LanguageChoice::Rust,
        Language::Python { .. } => LanguageChoice::Python,
        Language::Go { .. } => LanguageChoice::Go,
        Language::Java { .. } => LanguageChoice::Java,
        Language::JavaScript { .. } => LanguageChoice::JavaScript,
    }
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
        }
        | Language::Python {
            version: Some(version),
            ..
        }
        | Language::Go {
            version: Some(version),
            ..
        } => Some(format!("detected version: {version}")),
        Language::Java {
            jdk_package: Some(pkg),
            gradle_enable,
            maven_enable,
            ..
        } => {
            let jdk_info = if let Some(ver) = pkg.strip_prefix("pkgs.jdk") {
                format!("JDK {ver}")
            } else {
                pkg.clone()
            };
            let build_tool = match (gradle_enable, maven_enable) {
                (Some(true), _) => ", gradle",
                (_, Some(true)) => ", maven",
                _ => "",
            };
            Some(format!("detected {jdk_info}{build_tool}"))
        }
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

fn prompt_rust_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("Use default Rust config?")
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

fn prompt_python_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("Use default Python config?")
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

    let fetched = version_fetch::fetch_python_versions();
    let mut version_options: Vec<String> = vec!["default".to_string()];
    version_options.extend(fetched);
    let version_refs: Vec<&str> = version_options.iter().map(String::as_str).collect();
    let ver_idx = Select::with_theme(&theme)
        .with_prompt("Python version")
        .default(0)
        .items(&version_refs)
        .interact()
        .expect("interact err exit");
    let version = if ver_idx == 0 {
        None
    } else {
        Some(version_options[ver_idx].clone())
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
        package: None,
        uv_enable,
        venv_enable,
        venv_quiet,
    }
}

fn prompt_go_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("Use default Go config?")
        .default(true)
        .interact()
        .expect("interact err exit");
    if use_default {
        return Language::Go {
            version: None,
            package: None,
        };
    }

    let fetched = version_fetch::fetch_go_versions();
    let mut version_options: Vec<String> = vec!["default".to_string()];
    version_options.extend(fetched);
    let version_refs: Vec<&str> = version_options.iter().map(String::as_str).collect();
    let ver_idx = Select::with_theme(&theme)
        .with_prompt("Go version")
        .default(0)
        .items(&version_refs)
        .interact()
        .expect("interact err exit");
    let version = if ver_idx == 0 {
        None
    } else {
        Some(version_options[ver_idx].clone())
    };

    Language::Go {
        version,
        package: None,
    }
}

fn prompt_java_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("Use default Java config?")
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

    let fetched = version_fetch::fetch_jdk_versions();
    let mut jdk_options: Vec<String> = vec!["default".to_string()];
    jdk_options.extend(fetched);
    let jdk_refs: Vec<&str> = jdk_options.iter().map(String::as_str).collect();
    let jdk_idx = Select::with_theme(&theme)
        .with_prompt("JDK version")
        .default(0)
        .items(&jdk_refs)
        .interact()
        .expect("interact err exit");
    let jdk_package = if jdk_idx == 0 {
        None
    } else {
        Some(format!("pkgs.jdk{}", jdk_options[jdk_idx]))
    };

    let build_tools = vec!["None", "Gradle", "Maven"];
    let build_idx = Select::with_theme(&theme)
        .with_prompt("Build tool")
        .default(0)
        .items(&build_tools)
        .interact()
        .expect("interact err exit");
    let (gradle_enable, maven_enable) = match build_idx {
        1 => (Some(true), None),
        2 => (None, Some(true)),
        _ => (None, None),
    };

    Language::Java {
        jdk_package,
        gradle_enable,
        maven_enable,
    }
}

fn prompt_javascript_config() -> Language {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("Use default JavaScript config?")
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

    let fetched = version_fetch::fetch_node_versions();
    let mut node_options: Vec<String> = vec!["default".to_string()];
    node_options.extend(fetched);
    let node_refs: Vec<&str> = node_options.iter().map(String::as_str).collect();
    let node_idx = Select::with_theme(&theme)
        .with_prompt("Node.js version")
        .default(0)
        .items(&node_refs)
        .interact()
        .expect("interact err exit");
    let package = if node_idx == 0 {
        None
    } else {
        Some(format!("pkgs.nodejs_{}", node_options[node_idx]))
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

/// Prompts the user to select services.
///
/// # Panics
///
/// Panics if the terminal interaction fails.
#[must_use]
pub fn prompt_service_choices() -> Vec<ServiceChoice> {
    let options = vec!["PostgreSQL", "Redis", "MySQL"];
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select services (optional, press Enter to skip)")
        .items(&options)
        .interact()
        .expect("select err");

    selections
        .iter()
        .map(|&i| match i {
            0 => ServiceChoice::Postgres,
            1 => ServiceChoice::Redis,
            2 => ServiceChoice::Mysql,
            _ => unreachable!(),
        })
        .collect()
}

#[must_use]
pub fn prompt_service_config(choice: ServiceChoice) -> Service {
    match choice {
        ServiceChoice::Postgres => prompt_postgres_config(),
        ServiceChoice::Redis => Service::Redis,
        ServiceChoice::Mysql => prompt_mysql_config(),
    }
}

/// Displays detected services and prompts the user to confirm selections.
///
/// # Panics
///
/// Panics if the terminal interaction fails.
#[must_use]
pub fn confirm_detected_services(candidates: &[ServiceCandidate]) -> Vec<usize> {
    println!("\nDetected services:");
    for candidate in candidates {
        let name = service_display_name(&candidate.service);
        for reason in &candidate.reasons {
            println!("  {name} - {reason}");
        }
    }
    println!();

    let labels: Vec<String> = candidates
        .iter()
        .map(|c| service_display_name(&c.service).to_string())
        .collect();
    let defaults: Vec<bool> = vec![true; candidates.len()];

    MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Use detected services?")
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .expect("interact err")
}

fn service_display_name(service: &Service) -> &'static str {
    match service {
        Service::Postgres { .. } => "PostgreSQL",
        Service::Redis => "Redis",
        Service::Mysql { .. } => "MySQL",
    }
}

fn prompt_postgres_config() -> Service {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("Use default PostgreSQL config?")
        .default(true)
        .interact()
        .expect("interact err exit");
    if use_default {
        return Service::Postgres { package: None };
    }

    let packages = vec![
        "default",
        "pkgs.postgresql_14",
        "pkgs.postgresql_15",
        "pkgs.postgresql_16",
        "pkgs.postgresql_17",
    ];
    let pkg_idx = Select::with_theme(&theme)
        .with_prompt("PostgreSQL package")
        .default(0)
        .items(&packages)
        .interact()
        .expect("interact err exit");
    let package = if pkg_idx == 0 {
        None
    } else {
        Some(packages[pkg_idx].to_string())
    };

    Service::Postgres { package }
}

fn prompt_mysql_config() -> Service {
    let theme = ColorfulTheme::default();
    let use_default = Confirm::with_theme(&theme)
        .with_prompt("Use default MySQL config?")
        .default(true)
        .interact()
        .expect("interact err exit");
    if use_default {
        return Service::Mysql { package: None };
    }

    let packages = vec!["default (MariaDB)", "pkgs.mysql80"];
    let pkg_idx = Select::with_theme(&theme)
        .with_prompt("MySQL package")
        .default(0)
        .items(&packages)
        .interact()
        .expect("interact err exit");
    let package = if pkg_idx == 0 {
        None
    } else {
        Some(packages[pkg_idx].to_string())
    };

    Service::Mysql { package }
}
