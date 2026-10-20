use std::{
    fs, io,
    path::PathBuf,
    sync::{
        RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use serde::Serialize;
use tokio::sync::watch;

use crate::{
    anim,
    config::{Config, Lang},
    media::NowPlaying,
    theme,
};

pub struct Cover {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

/// Aparência enviada ao widget pelo SSE.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WidgetStyle {
    pub width: u16,
    pub bg: String,
    pub text: String,
    pub accent: String,
    pub bg_opacity: u8,
    /// Revisão da animação em `/animation`; muda quando o arquivo é trocado.
    pub anim: u64,
    pub lang: Lang,
}

pub struct AppState {
    pub config_path: PathBuf,
    pub assets_dir: PathBuf,
    /// Porta em que o servidor realmente escutou (a do config só vale após reiniciar).
    pub port: u16,
    pub now: watch::Sender<NowPlaying>,
    pub style: watch::Sender<WidgetStyle>,
    pub cover: RwLock<Option<Cover>>,
    config: RwLock<Config>,
    anim_rev: AtomicU64,
}

impl AppState {
    pub fn new(config: Config, config_path: PathBuf, assets_dir: PathBuf, port: u16) -> Self {
        let style = watch::Sender::new(style_of(&config, 1));
        Self {
            config_path,
            assets_dir,
            port,
            now: watch::Sender::new(NowPlaying::Hidden),
            style,
            cover: RwLock::new(None),
            config: RwLock::new(config),
            anim_rev: AtomicU64::new(1),
        }
    }

    pub fn config(&self) -> Config {
        self.config.read().unwrap().clone()
    }

    /// Valida, salva no disco e publica a nova config para os widgets abertos.
    pub fn apply_config(&self, new: Config) -> io::Result<Config> {
        let mut new = new.sanitized();
        if !new.animation.is_empty() && !self.assets_dir.join(&new.animation).is_file() {
            new.animation.clear();
        }
        new.save(&self.config_path)?;
        let changed_animation = {
            let mut current = self.config.write().unwrap();
            let changed = current.animation != new.animation;
            *current = new.clone();
            changed
        };
        if changed_animation {
            self.anim_rev.fetch_add(1, Ordering::Relaxed);
        }
        self.publish_style();
        Ok(new)
    }

    /// Salva um upload em `assets/` e passa a usá-lo como animação.
    pub fn store_animation(&self, original_name: &str, format: anim::Format, bytes: &[u8]) -> io::Result<Config> {
        let name = anim::safe_file_name(original_name, format);
        fs::create_dir_all(&self.assets_dir)?;
        fs::write(self.assets_dir.join(&name), bytes)?;
        // Mesmo nome sobrescrito: força o widget a recarregar o arquivo.
        self.anim_rev.fetch_add(1, Ordering::Relaxed);
        self.apply_config(Config { animation: name, ..self.config() })
    }

    /// Bytes e tipo da animação atual; cai no pinguim embutido se o arquivo sumir.
    pub fn animation(&self) -> (Vec<u8>, &'static str) {
        let name = self.config().animation;
        if let Some(format) = anim::format_of_name(&name)
            && let Ok(bytes) = fs::read(self.assets_dir.join(&name))
        {
            return (bytes, format.mime());
        }
        (anim::DEFAULT_GIF.to_vec(), anim::Format::Gif.mime())
    }

    fn publish_style(&self) {
        let rev = self.anim_rev.load(Ordering::Relaxed);
        self.style.send_replace(style_of(&self.config(), rev));
    }
}

fn style_of(config: &Config, anim_rev: u64) -> WidgetStyle {
    let colors = theme::resolve(config);
    WidgetStyle {
        width: config.width,
        bg: colors.bg,
        text: colors.text,
        accent: colors.accent,
        bg_opacity: config.bg_opacity,
        anim: anim_rev,
        lang: config.lang,
    }
}
