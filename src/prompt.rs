use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};

use crate::schema::Language;

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
