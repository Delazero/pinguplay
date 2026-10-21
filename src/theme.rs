use serde::Serialize;

use crate::config::Config;

pub const CUSTOM: &str = "custom";
/// Tema de instalações novas.
pub const DEFAULT_ID: &str = "dark";

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub bg: &'static str,
    pub text: &'static str,
    pub accent: &'static str,
}

pub const PRESETS: &[Preset] = &[
    Preset { id: "pink", bg: "#ffd6e7", text: "#5a2a3f", accent: "#ff6fa5" },
    Preset { id: "white", bg: "#ffffff", text: "#2b2b2f", accent: "#9a9aa5" },
    Preset { id: "light_gray", bg: "#e9e9ee", text: "#33333d", accent: "#8a8a99" },
    Preset { id: "dark", bg: "#1e1e24", text: "#f2f2f5", accent: "#b9a7ff" },
    Preset { id: "lilac", bg: "#e6dcff", text: "#3d2f66", accent: "#9b7bff" },
    Preset { id: "mint", bg: "#d4f5e6", text: "#1f4d3a", accent: "#3cc28c" },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Colors {
    pub bg: String,
    pub text: String,
    pub accent: String,
}

pub fn preset(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id)
}

pub fn is_known(id: &str) -> bool {
    id == CUSTOM || preset(id).is_some()
}

/// Resolve as cores finais do widget a partir do tema escolhido.
pub fn resolve(config: &Config) -> Colors {
    if config.theme == CUSTOM {
        return Colors {
            bg: config.custom_bg.clone(),
            text: config.custom_text.clone(),
            accent: config.custom_accent.clone(),
        };
    }
    let preset = preset(&config.theme)
        .or_else(|| preset(DEFAULT_ID))
        .expect("default theme exists");
    Colors {
        bg: preset.bg.into(),
        text: preset.text.into(),
        accent: preset.accent.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_preset_colors() {
        let config = Config { theme: "dark".into(), ..Config::default() };
        assert_eq!(resolve(&config).bg, "#1e1e24");
    }

    #[test]
    fn resolves_custom_colors() {
        let config = Config {
            theme: CUSTOM.into(),
            custom_bg: "#010203".into(),
            custom_text: "#040506".into(),
            custom_accent: "#070809".into(),
            ..Config::default()
        };
        assert_eq!(
            resolve(&config),
            Colors { bg: "#010203".into(), text: "#040506".into(), accent: "#070809".into() }
        );
    }

    #[test]
    fn unknown_theme_falls_back_to_default() {
        let config = Config { theme: "nope".into(), ..Config::default() };
        assert_eq!(resolve(&config).bg, preset(DEFAULT_ID).unwrap().bg);
    }

    #[test]
    fn knows_presets_and_custom() {
        assert!(is_known("mint"));
        assert!(is_known(CUSTOM));
        assert!(!is_known("rainbow"));
    }
}
