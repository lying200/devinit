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
