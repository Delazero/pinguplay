// Sem janela de console no build de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod anim;
mod autostart;
mod config;
mod i18n;
mod media;
mod server;
mod state;
mod theme;
mod tray;
mod watcher;

use std::{fs, net::TcpListener, path::PathBuf, sync::Arc};

use windows::{
    Win32::UI::WindowsAndMessaging::{MB_ICONWARNING, MB_OK, MessageBoxW},
    core::{HSTRING, w},
};

use crate::{config::Config, i18n::Text, state::AppState};

fn main() {
    // Modo portátil: config e assets ficam ao lado do .exe.
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    let config_path = exe_dir.join("config.toml");
    let assets_dir = exe_dir.join("assets");
    let _ = fs::create_dir_all(&assets_dir);

    let config = Config::load(&config_path);
    if !config_path.exists() {
        let _ = config.save(&config_path);
    }
    let (lang, port) = (config.lang, config.port);

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => listener,
        Err(_) => {
            let message = i18n::text(lang, Text::PortBusy).replace("{port}", &port.to_string());
            // SAFETY: MessageBoxW com strings válidas e sem janela pai.
            unsafe { MessageBoxW(None, &HSTRING::from(message), w!("pinguplay"), MB_OK | MB_ICONWARNING) };
            return;
        }
    };
    listener.set_nonblocking(true).expect("failed to configure listener");

    let state = Arc::new(AppState::new(config, config_path, assets_dir, port));
    watcher::spawn(state.clone());

    let server_state = state.clone();
    std::thread::Builder::new()
        .name("http".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("failed to start tokio runtime");
            if let Err(err) = runtime.block_on(server::serve(listener, server_state)) {
                eprintln!("server stopped: {err}");
            }
        })
        .expect("failed to spawn http thread");

    tray::run(lang, port);
}
