use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::{anim, theme};

pub const DEFAULT_PORT: u16 = 7878;
pub const MIN_WIDTH: u16 = 260;
pub const MAX_WIDTH: u16 = 800;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lang {
    #[default]
    Pt,
    En,
}

/// De qual player o widget lê a música.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    #[default]
    Spotify,
    Any,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub lang: Lang,
    pub port: u16,
    pub source: Source,
    pub theme: String,
    pub custom_bg: String,
    pub custom_text: String,
    pub custom_accent: String,
    /// Opacidade do fundo do card, 0–100.
    pub bg_opacity: u8,
    pub width: u16,
    /// Arquivo dentro de `assets/`. Vazio = pinguim padrão embutido.
    pub animation: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lang: Lang::Pt,
            port: DEFAULT_PORT,
            source: Source::Spotify,
            theme: theme::PRESETS[0].id.into(),
            custom_bg: "#ffd6e7".into(),
            custom_text: "#5a2a3f".into(),
            custom_accent: "#ff6fa5".into(),
            bg_opacity: 100,
            width: 420,
            animation: String::new(),
        }
    }
}

impl Config {
    /// Corrige valores fora do esperado (arquivo editado à mão, request malformado).
    pub fn sanitized(mut self) -> Self {
        let defaults = Config::default();
        if self.port == 0 {
            self.port = defaults.port;
        }
        if !theme::is_known(&self.theme) {
            self.theme = defaults.theme;
        }
        for (value, fallback) in [
            (&mut self.custom_bg, defaults.custom_bg),
            (&mut self.custom_text, defaults.custom_text),
            (&mut self.custom_accent, defaults.custom_accent),
        ] {
            if is_hex_color(value) {
                *value = value.to_ascii_lowercase();
            } else {
                *value = fallback;
            }
        }
        self.bg_opacity = self.bg_opacity.min(100);
        self.width = self.width.clamp(MIN_WIDTH, MAX_WIDTH);
        if !self.animation.is_empty() && !anim::is_safe_name(&self.animation) {
            self.animation.clear();
        }
        self
    }

    /// Lê o `config.toml`. Arquivo ausente ou ilegível vira config padrão;
    /// um campo inválido volta só ele ao padrão.
    pub fn load(path: &Path) -> Config {
        fs::read_to_string(path)
            .map(|text| Config::from_toml_lenient(&text))
            .unwrap_or_default()
            .sanitized()
    }

    fn from_toml_lenient(text: &str) -> Config {
        let Ok(file) = text.parse::<toml::Table>() else {
            return Config::default();
        };
        let Some(mut merged) = toml::Value::try_from(Config::default())
            .ok()
            .and_then(|v| v.as_table().cloned())
        else {
            return Config::default();
        };
        // Aplica campo a campo e descarta o que não desserializa (ex.: lang = "es").
        for (key, value) in file {
            if !merged.contains_key(&key) {
                continue;
            }
            let mut candidate = merged.clone();
            candidate.insert(key, value);
            if toml::Value::Table(candidate.clone()).try_into::<Config>().is_ok() {
                merged = candidate;
            }
        }
        toml::Value::Table(merged).try_into().unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let text = toml::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, text)
    }
}

pub fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_color_validation() {
        assert!(is_hex_color("#a1B2c3"));
        assert!(!is_hex_color("a1b2c3"));
        assert!(!is_hex_color("#a1b2c"));
        assert!(!is_hex_color("#gggggg"));
    }

    #[test]
    fn sanitize_clamps_and_fixes_invalid_values() {
        let config = Config {
            port: 0,
            theme: "rainbow".into(),
            custom_bg: "red".into(),
            custom_text: "#ABCDEF".into(),
            bg_opacity: 250,
            width: 5000,
            animation: "../../evil.gif".into(),
            ..Config::default()
        }
        .sanitized();
        let defaults = Config::default();
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.theme, defaults.theme);
        assert_eq!(config.custom_bg, defaults.custom_bg);
        assert_eq!(config.custom_text, "#abcdef");
        assert_eq!(config.bg_opacity, 100);
        assert_eq!(config.width, MAX_WIDTH);
        assert_eq!(config.animation, "");
    }

    #[test]
    fn partial_toml_fills_defaults() {
        let config: Config = toml::from_str("theme = \"mint\"\nwidth = 300").unwrap();
        assert_eq!(config.theme, "mint");
        assert_eq!(config.width, 300);
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.source, Source::Spotify);
    }

    #[test]
    fn save_then_load_roundtrips() {
        let dir = std::env::temp_dir().join(format!("pinguplay-config-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        let config = Config { lang: Lang::En, source: Source::Any, theme: "lilac".into(), ..Config::default() };
        config.save(&path).unwrap();
        assert_eq!(Config::load(&path), config);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn one_invalid_field_keeps_the_others() {
        let config = Config::from_toml_lenient("lang = \"es\"\ntheme = \"mint\"\nwidth = \"wide\"\nbg_opacity = 40");
        assert_eq!(config.lang, Lang::Pt);
        assert_eq!(config.theme, "mint");
        assert_eq!(config.width, Config::default().width);
        assert_eq!(config.bg_opacity, 40);
    }

    #[test]
    fn missing_or_broken_file_gives_defaults() {
        let dir = std::env::temp_dir().join(format!("pinguplay-broken-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        assert_eq!(Config::load(&path), Config::default());
        fs::write(&path, "this is = = not toml").unwrap();
        assert_eq!(Config::load(&path), Config::default());
        fs::remove_dir_all(&dir).unwrap();
    }
}
