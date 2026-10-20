//! Lê a música atual pela API de mídia do Windows (GSMTC) e publica no estado.

use std::{
    sync::Arc,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use windows::{
    Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionMediaProperties as MediaProperties,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
    },
    Storage::Streams::DataReader,
    Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize},
};

use crate::{
    config::Source,
    media::{self, NowPlaying, Sample, SessionInfo, Status},
    state::{AppState, Cover},
};

const POLL: Duration = Duration::from_millis(500);
/// O Spotify às vezes entrega a capa nova alguns instantes depois do título:
/// rebusca a capa nos polls 0, 2 e 4 após a troca de faixa.
const COVER_REFRESH_POLLS: u32 = 4;
const MAX_COVER_BYTES: u32 = 8 * 1024 * 1024;

pub fn spawn(state: Arc<AppState>) {
    thread::Builder::new()
        .name("media-watcher".into())
        .spawn(move || {
            // SAFETY: inicializa o apartment WinRT desta thread antes de qualquer chamada.
            let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
            Watcher::default().run(&state);
        })
        .expect("failed to spawn media watcher thread");
}

#[derive(Default)]
struct Watcher {
    manager: Option<Manager>,
    last: NowPlaying,
    last_at: u64,
    track_key: String,
    polls_since_change: u32,
    cover_rev: u64,
    track_cover: u64,
}

struct Current {
    sample: Sample,
    props: MediaProperties,
}

impl Watcher {
    fn run(&mut self, state: &AppState) -> ! {
        loop {
            self.tick(state);
            thread::sleep(POLL);
        }
    }

    fn tick(&mut self, state: &AppState) {
        let source = state.config().source;
        let now = now_ms();
        let current = match self.read(source, now) {
            Ok(current) => current,
            Err(_) => {
                // Manager inválido (ex.: Explorer reiniciou): recria no próximo poll.
                self.manager = None;
                None
            }
        };

        let next = match &current {
            Some(c) if !media::is_ad(&c.sample) => {
                self.update_cover(state, c);
                media::to_now_playing(Some(&c.sample), self.track_cover)
            }
            other => {
                self.track_key.clear();
                media::to_now_playing(other.as_ref().map(|c| &c.sample), 0)
            }
        };

        if media::should_emit(&self.last, self.last_at, &next, now) {
            state.now.send_replace(next.clone());
            self.last = next;
            self.last_at = now;
        }
    }

    fn update_cover(&mut self, state: &AppState, current: &Current) {
        let key = format!("{}\n{}\n{}", current.sample.app_id, current.sample.title, current.sample.artist);
        if key != self.track_key {
            self.track_key = key;
            self.polls_since_change = 0;
            self.track_cover = 0;
        }
        let poll = self.polls_since_change;
        self.polls_since_change = poll.saturating_add(1);
        if poll > COVER_REFRESH_POLLS || !poll.is_multiple_of(2) {
            return;
        }
        let Some((bytes, content_type)) = read_thumbnail(&current.props) else {
            return;
        };
        let mut cover = state.cover.write().unwrap();
        let same = cover.as_ref().is_some_and(|c| c.bytes == bytes);
        if !same {
            self.cover_rev += 1;
            *cover = Some(Cover { bytes, content_type });
        }
        self.track_cover = self.cover_rev;
    }

    fn read(&mut self, source: Source, now: u64) -> windows::core::Result<Option<Current>> {
        if self.manager.is_none() {
            self.manager = Some(Manager::RequestAsync()?.join()?);
        }
        let manager = self.manager.as_ref().unwrap();
        let sessions = manager.GetSessions()?;

        let mut infos = Vec::new();
        let mut handles = Vec::new();
        for i in 0..sessions.Size()? {
            let session = sessions.GetAt(i)?;
            let status = match session.GetPlaybackInfo()?.PlaybackStatus()? {
                PlaybackStatus::Playing | PlaybackStatus::Changing => Status::Playing,
                PlaybackStatus::Paused | PlaybackStatus::Stopped => Status::Paused,
                _ => Status::Inactive,
            };
            infos.push(SessionInfo { app_id: session.SourceAppUserModelId()?.to_string(), status });
            handles.push(session);
        }
        let Some(index) = media::pick_session(&infos, source) else {
            return Ok(None);
        };
        let session = &handles[index];
        let info = &infos[index];

        let props = session.TryGetMediaPropertiesAsync()?.join()?;
        let timeline = session.GetTimelineProperties()?;
        let start = timeline.StartTime()?.Duration;
        let duration_ms = media::ticks_to_ms(timeline.EndTime()?.Duration - start);
        let position_ms = media::ticks_to_ms(timeline.Position()?.Duration - start);
        let updated_ms = media::winrt_time_to_unix_ms(timeline.LastUpdatedTime()?.UniversalTime);
        let playing = info.status == Status::Playing;

        Ok(Some(Current {
            sample: Sample {
                app_id: info.app_id.clone(),
                playing,
                title: props.Title()?.to_string(),
                artist: props.Artist()?.to_string(),
                position_ms: media::interpolate(position_ms, duration_ms, playing, now as i64 - updated_ms),
                duration_ms,
            },
            props,
        }))
    }
}

fn read_thumbnail(props: &MediaProperties) -> Option<(Vec<u8>, String)> {
    let stream = props.Thumbnail().ok()?.OpenReadAsync().ok()?.join().ok()?;
    let size = u32::try_from(stream.Size().ok()?).ok()?;
    if size == 0 || size > MAX_COVER_BYTES {
        return None;
    }
    let content_type = stream
        .ContentType()
        .map(|c| c.to_string())
        .ok()
        .filter(|c| c.starts_with("image/"))
        .unwrap_or_else(|| "image/jpeg".into());
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0).ok()?).ok()?;
    reader.LoadAsync(size).ok()?.join().ok()?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes).ok()?;
    Some((bytes, content_type))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}
