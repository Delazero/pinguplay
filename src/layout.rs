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
    /// Tamanho do texto em % (100 = original).
    pub text_scale: u16,
    /// Altura do card em px, já considerando o tamanho do texto.
    pub height: u16,
}

/// Tamanho recomendado para o Browser Source do OBS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ObsSize {
    pub width: u16,
    pub height: u16,
}

// Métricas do card em web/widget.html a 100%: manter em sincronia com o CSS.
const LINE_HEIGHT: f32 = 1.3;
const TITLE_PX: f32 = 17.0;
const ARTIST_PX: f32 = 14.0;
const COMPACT_TITLE_PX: f32 = 16.0;
/// Barra: 6px de margem + 4px de altura (compacto: 4 + 3).
const BAR_PX: f32 = 10.0;
const COMPACT_BAR_PX: f32 = 7.0;
/// Padding vertical do card (compacto: respiro mínimo em volta do texto).
const PAD_Y_PX: f32 = 16.0;
const COMPACT_PAD_Y_PX: f32 = 20.0;
/// Folga para arredondamento de fonte (DPI, versão do Chromium do OBS): sem ela sobra ~1px.
/// O compacto não precisa: o respiro dele já é generoso.
const SLACK_PX: f32 = 2.0;
/// Altura mínima: cabe o pinguim de 64px (compacto: pílula baixa).
const MIN_HEIGHT: u16 = 80;
const COMPACT_MIN_HEIGHT: u16 = 48;
/// Padding de 8px do body do widget em volta do card.
const WIDGET_MARGIN: u16 = 16;

/// O card só cresce quando o texto deixa de caber na altura mínima.
fn card_height(compact: bool, artist: bool, progress: bool, text_scale: u16) -> u16 {
    let scale = f32::from(text_scale) / 100.0;
    let (text, bar, pad, min) = if compact {
        (COMPACT_TITLE_PX * LINE_HEIGHT * scale, COMPACT_BAR_PX, COMPACT_PAD_Y_PX, COMPACT_MIN_HEIGHT)
    } else {
        let artist_px = if artist { ARTIST_PX * LINE_HEIGHT * scale } else { 0.0 };
        (TITLE_PX * LINE_HEIGHT * scale + artist_px, BAR_PX, PAD_Y_PX + SLACK_PX, MIN_HEIGHT)
    };
    let needed = text + if progress { bar } else { 0.0 } + pad;
    (needed.ceil() as u16).max(min)
}

pub fn obs_size(config: &Config, card: &CardLayout) -> ObsSize {
    ObsSize { width: config.width + WIDGET_MARGIN, height: card.height + WIDGET_MARGIN }
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
    let compact = !animation && !cover;
    CardLayout {
        animation,
        cover,
        artist,
        progress,
        compact,
        pause,
        text_scale: config.text_scale,
        height: card_height(compact, artist, progress, config.text_scale),
    }
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

    fn scaled(layout: Layout, text_scale: u16) -> CardLayout {
        resolve(&Config { layout, text_scale, ..Config::default() })
    }

    #[test]
    fn default_text_size_keeps_original_heights() {
        assert_eq!(scaled(Layout::Full, 100).height, 80);
        assert_eq!(scaled(Layout::NoGif, 100).height, 80);
        assert_eq!(scaled(Layout::Minimal, 100).height, 48);
        assert_eq!(custom(false, false, true, true).height, 48);
    }

    #[test]
    fn smaller_text_never_shrinks_the_card() {
        assert_eq!(scaled(Layout::Full, 80).height, 80);
        assert_eq!(scaled(Layout::Minimal, 80).height, 48);
    }

    #[test]
    fn card_grows_only_when_text_no_longer_fits() {
        assert_eq!(scaled(Layout::Full, 125).height, 80);
        assert!(scaled(Layout::Full, 135).height > 80);
        assert_eq!(scaled(Layout::Full, 160).height, 93);
        assert_eq!(scaled(Layout::Minimal, 160).height, 54);
    }

    #[test]
    fn hidden_parts_leave_more_room_for_text() {
        let at_160 = |show_artist| {
            resolve(&Config { layout: Layout::Custom, show_artist, text_scale: 160, ..Config::default() })
        };
        assert_eq!(at_160(true).height, 93);
        assert_eq!(at_160(false).height, 80);
    }

    #[test]
    fn card_exposes_text_scale_for_the_widget() {
        assert_eq!(scaled(Layout::Full, 125).text_scale, 125);
    }

    #[test]
    fn obs_size_adds_widget_margin() {
        let config = Config { width: 420, layout: Layout::Minimal, ..Config::default() };
        assert_eq!(obs_size(&config, &resolve(&config)), ObsSize { width: 436, height: 64 });
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
