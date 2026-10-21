//! Quais partes do card aparecem e como ele sinaliza pausa, por layout.

use serde::Serialize;

use crate::config::{Config, Layout};

/// Como o card mostra que a música está pausada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PauseCue {
    /// Pinguim congela no primeiro frame.
    Freeze,
    /// Ícone ⏸ sobre a capa.
    Icon,
    /// Card inteiro fica translúcido.
    Dim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CardLayout {
    pub animation: bool,
    pub cover: bool,
    pub artist: bool,
    pub progress: bool,
    /// Sem imagem nenhuma: pílula baixa, título e artista na mesma linha.
    pub compact: bool,
    pub pause: PauseCue,
}

/// Os layouts prontos são atalhos para combinações do personalizado.
pub fn resolve(config: &Config) -> CardLayout {
    let (animation, cover, artist, progress) = match config.layout {
        Layout::Full => (true, true, true, true),
        Layout::NoGif => (false, true, true, true),
        Layout::Minimal => (false, false, true, false),
        Layout::Custom => (config.show_animation, config.show_cover, config.show_artist, config.show_progress),
    };
    let pause = if animation {
        PauseCue::Freeze
    } else if cover {
        PauseCue::Icon
    } else {
        PauseCue::Dim
    };
    CardLayout { animation, cover, artist, progress, compact: !animation && !cover, pause }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_layout(layout: Layout) -> Config {
        Config { layout, ..Config::default() }
    }

    fn custom(animation: bool, cover: bool, artist: bool, progress: bool) -> CardLayout {
        resolve(&Config {
            layout: Layout::Custom,
            show_animation: animation,
            show_cover: cover,
            show_artist: artist,
            show_progress: progress,
            ..Config::default()
        })
    }

    #[test]
    fn full_shows_everything_and_freezes_penguin() {
        let card = resolve(&with_layout(Layout::Full));
        assert!(card.animation && card.cover && card.artist && card.progress);
        assert!(!card.compact);
        assert_eq!(card.pause, PauseCue::Freeze);
    }

    #[test]
    fn no_gif_hides_penguin_and_pauses_with_icon() {
        let card = resolve(&with_layout(Layout::NoGif));
        assert!(!card.animation && card.cover && card.artist && card.progress);
        assert!(!card.compact);
        assert_eq!(card.pause, PauseCue::Icon);
    }

    #[test]
    fn minimal_is_compact_text_only_and_dims_on_pause() {
        let card = resolve(&with_layout(Layout::Minimal));
        assert!(!card.animation && !card.cover && card.artist && !card.progress);
        assert!(card.compact);
        assert_eq!(card.pause, PauseCue::Dim);
    }

    #[test]
    fn presets_ignore_custom_toggles() {
        let config = Config { layout: Layout::Full, show_cover: false, ..Config::default() };
        assert!(resolve(&config).cover);
    }

    #[test]
    fn custom_follows_toggles_and_derives_shape_and_pause() {
        let penguin_text = custom(true, false, true, false);
        assert!(!penguin_text.compact);
        assert_eq!(penguin_text.pause, PauseCue::Freeze);

        let compact_with_bar = custom(false, false, false, true);
        assert!(compact_with_bar.compact && compact_with_bar.progress && !compact_with_bar.artist);
        assert_eq!(compact_with_bar.pause, PauseCue::Dim);

        assert_eq!(custom(false, true, false, false).pause, PauseCue::Icon);
    }
}
