//! Regras puras sobre o que o widget mostra, independentes da API do Windows.

use serde::Serialize;

use crate::config::Source;

/// Diferença entre a época do Windows (1601) e a Unix (1970), em unidades de 100 ns.
const WINDOWS_TO_UNIX_EPOCH_100NS: i64 = 116_444_736_000_000_000;
/// Desvio de posição que conta como seek (e não só atraso de polling).
const SEEK_TOLERANCE_MS: i64 = 2_000;
/// Anúncios de áudio do Spotify duram até ~60 s.
const MAX_AD_MS: u64 = 65_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Playing,
    Paused,
    Inactive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub app_id: String,
    pub status: Status,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample {
    pub app_id: String,
    pub playing: bool,
    pub title: String,
    pub artist: String,
    pub position_ms: u64,
    pub duration_ms: u64,
}

/// Estado enviado ao widget.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NowPlaying {
    /// Player fechado: o card some.
    #[default]
    Hidden,
    /// Anúncio do Spotify: "Intervalo ♪" com pinguim parado.
    Ad,
    Track {
        title: String,
        artist: String,
        playing: bool,
        position_ms: u64,
        duration_ms: u64,
        /// Revisão da capa em `/cover`; 0 = sem capa.
        cover: u64,
    },
}

pub fn is_spotify(app_id: &str) -> bool {
    app_id.to_ascii_lowercase().contains("spotify")
}

/// Escolhe a sessão a exibir: a que está tocando tem prioridade sobre a pausada.
pub fn pick_session(sessions: &[SessionInfo], source: Source) -> Option<usize> {
    let eligible = |s: &SessionInfo| match source {
        Source::Spotify => is_spotify(&s.app_id),
        Source::Any => true,
    };
    let find = |status| {
        sessions
            .iter()
            .position(|s| s.status == status && eligible(s))
    };
    find(Status::Playing).or_else(|| find(Status::Paused))
}

/// O Spotify free anuncia propagandas como faixas sem artista ou com título "Advertisement".
/// Sem artista só conta como anúncio se for curto: arquivo local sem tag é música normal.
pub fn is_ad(sample: &Sample) -> bool {
    if !is_spotify(&sample.app_id) {
        return false;
    }
    let title = sample.title.trim();
    title.is_empty()
        || title.eq_ignore_ascii_case("advertisement")
        || title.eq_ignore_ascii_case("spotify")
        || (sample.artist.trim().is_empty() && sample.duration_ms <= MAX_AD_MS)
}

pub fn to_now_playing(sample: Option<&Sample>, cover: u64) -> NowPlaying {
    match sample {
        None => NowPlaying::Hidden,
        Some(s) if is_ad(s) => NowPlaying::Ad,
        Some(s) => NowPlaying::Track {
            title: s.title.trim().into(),
            artist: s.artist.trim().into(),
            playing: s.playing,
            position_ms: s.position_ms,
            duration_ms: s.duration_ms,
            cover,
        },
    }
}

pub fn winrt_time_to_unix_ms(universal_time: i64) -> i64 {
    (universal_time - WINDOWS_TO_UNIX_EPOCH_100NS) / 10_000
}

pub fn ticks_to_ms(ticks: i64) -> u64 {
    (ticks.max(0) / 10_000) as u64
}

/// Posição atual estimada: o Windows só atualiza a timeline de vez em quando.
pub fn interpolate(position_ms: u64, duration_ms: u64, playing: bool, elapsed_ms: i64) -> u64 {
    let mut position = position_ms;
    if playing && elapsed_ms > 0 {
        position = position.saturating_add(elapsed_ms as u64);
    }
    if duration_ms > 0 {
        position = position.min(duration_ms);
    }
    position
}

/// Decide se o novo estado merece ser enviado ao widget.
/// Avanço normal da posição não conta: o widget interpola sozinho.
pub fn should_emit(prev: &NowPlaying, prev_at_ms: u64, next: &NowPlaying, next_at_ms: u64) -> bool {
    match (prev, next) {
        (
            NowPlaying::Track { title: t1, artist: a1, playing: p1, position_ms: pos1, duration_ms: d1, cover: c1 },
            NowPlaying::Track { title: t2, artist: a2, playing: p2, position_ms: pos2, duration_ms: d2, cover: c2 },
        ) => {
            if (t1, a1, p1, d1, c1) != (t2, a2, p2, d2, c2) {
                return true;
            }
            let elapsed = if *p1 { next_at_ms.saturating_sub(prev_at_ms) } else { 0 };
            let expected = interpolate(*pos1, *d1, *p1, elapsed as i64) as i64;
            (*pos2 as i64 - expected).abs() > SEEK_TOLERANCE_MS
        }
        _ => prev != next,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(app_id: &str, status: Status) -> SessionInfo {
        SessionInfo { app_id: app_id.into(), status }
    }

    fn sample(title: &str, artist: &str) -> Sample {
        Sample {
            app_id: "Spotify.exe".into(),
            playing: true,
            title: title.into(),
            artist: artist.into(),
            position_ms: 1_000,
            duration_ms: 180_000,
        }
    }

    fn track(title: &str, playing: bool, position_ms: u64) -> NowPlaying {
        NowPlaying::Track {
            title: title.into(),
            artist: "Artist".into(),
            playing,
            position_ms,
            duration_ms: 200_000,
            cover: 1,
        }
    }

    #[test]
    fn detects_spotify_desktop_and_store_ids() {
        assert!(is_spotify("Spotify.exe"));
        assert!(is_spotify("SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify"));
        assert!(!is_spotify("chrome.exe"));
    }

    #[test]
    fn spotify_source_ignores_other_players() {
        let sessions = [session("chrome.exe", Status::Playing), session("Spotify.exe", Status::Paused)];
        assert_eq!(pick_session(&sessions, Source::Spotify), Some(1));
    }

    #[test]
    fn any_source_prefers_playing_session() {
        let sessions = [session("Spotify.exe", Status::Paused), session("chrome.exe", Status::Playing)];
        assert_eq!(pick_session(&sessions, Source::Any), Some(1));
    }

    #[test]
    fn inactive_sessions_are_hidden() {
        let sessions = [session("Spotify.exe", Status::Inactive)];
        assert_eq!(pick_session(&sessions, Source::Spotify), None);
        assert_eq!(pick_session(&[], Source::Any), None);
    }

    #[test]
    fn spotify_ads_are_detected() {
        assert!(is_ad(&sample("Advertisement", "")));
        assert!(is_ad(&sample("Spotify", "")));
        assert!(is_ad(&sample("", "Someone")));
        let short = Sample { duration_ms: 30_000, ..sample("Buy stuff", "") };
        assert!(is_ad(&short));
        let unknown_length = Sample { duration_ms: 0, ..sample("Buy stuff", "") };
        assert!(is_ad(&unknown_length));
        assert!(!is_ad(&sample("Song", "Artist")));
    }

    #[test]
    fn untagged_local_files_are_not_ads() {
        assert!(!is_ad(&sample("my demo track", "")));
    }

    #[test]
    fn other_players_never_count_as_ads() {
        let s = Sample { app_id: "chrome.exe".into(), ..sample("Video", "") };
        assert!(!is_ad(&s));
    }

    #[test]
    fn maps_samples_to_widget_state() {
        assert_eq!(to_now_playing(None, 0), NowPlaying::Hidden);
        assert_eq!(to_now_playing(Some(&sample("Advertisement", "")), 3), NowPlaying::Ad);
        assert_eq!(
            to_now_playing(Some(&sample(" Song ", "Artist")), 3),
            NowPlaying::Track {
                title: "Song".into(),
                artist: "Artist".into(),
                playing: true,
                position_ms: 1_000,
                duration_ms: 180_000,
                cover: 3,
            }
        );
    }

    #[test]
    fn converts_winrt_time() {
        assert_eq!(winrt_time_to_unix_ms(WINDOWS_TO_UNIX_EPOCH_100NS), 0);
        assert_eq!(winrt_time_to_unix_ms(WINDOWS_TO_UNIX_EPOCH_100NS + 15_000_000), 1_500);
        assert_eq!(ticks_to_ms(30_000_000), 3_000);
        assert_eq!(ticks_to_ms(-5), 0);
    }

    #[test]
    fn interpolates_only_while_playing() {
        assert_eq!(interpolate(10_000, 60_000, true, 2_500), 12_500);
        assert_eq!(interpolate(10_000, 60_000, false, 2_500), 10_000);
        assert_eq!(interpolate(59_000, 60_000, true, 5_000), 60_000);
        assert_eq!(interpolate(10_000, 0, true, 5_000), 15_000);
        assert_eq!(interpolate(10_000, 60_000, true, -400), 10_000);
    }

    #[test]
    fn emits_on_track_or_state_changes() {
        assert!(should_emit(&NowPlaying::Hidden, 0, &track("A", true, 0), 0));
        assert!(should_emit(&track("A", true, 0), 0, &track("B", true, 500), 500));
        assert!(should_emit(&track("A", true, 0), 0, &track("A", false, 500), 500));
        assert!(should_emit(&track("A", true, 0), 0, &NowPlaying::Ad, 500));
        assert!(!should_emit(&NowPlaying::Hidden, 0, &NowPlaying::Hidden, 500));
    }

    #[test]
    fn natural_progress_is_not_emitted_but_seeks_are() {
        assert!(!should_emit(&track("A", true, 10_000), 0, &track("A", true, 15_100), 5_000));
        assert!(should_emit(&track("A", true, 10_000), 0, &track("A", true, 60_000), 5_000));
        assert!(should_emit(&track("A", true, 10_000), 0, &track("A", true, 1_000), 5_000));
        assert!(!should_emit(&track("A", false, 10_000), 0, &track("A", false, 10_000), 5_000));
        assert!(should_emit(&track("A", false, 10_000), 0, &track("A", false, 40_000), 5_000));
    }
}
