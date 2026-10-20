//! Ícone na bandeja e loop de mensagens do Windows (roda na thread principal).

use std::sync::Arc;

use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use windows::{
    Win32::{
        Foundation::{LPARAM, WPARAM},
        System::Threading::GetCurrentThreadId,
        UI::{
            Shell::ShellExecuteW,
            WindowsAndMessaging::{
                DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, SW_SHOWNORMAL, TranslateMessage, WM_APP,
            },
        },
    },
    core::{HSTRING, w},
};

use crate::{
    autostart,
    config::Lang,
    i18n::{Text, text},
    state::AppState,
};

const ICON_SIZE: u32 = 32;
/// Mensagem de thread postada quando o idioma muda na página de config.
const WM_APP_LANG_CHANGED: u32 = WM_APP + 1;

struct Items {
    open: MenuItem,
    copy: MenuItem,
    startup: CheckMenuItem,
    quit: MenuItem,
}

impl Items {
    fn new(lang: Lang) -> Self {
        let items = Self {
            open: MenuItem::new("", true, None),
            copy: MenuItem::new("", true, None),
            startup: CheckMenuItem::new("", true, autostart::is_enabled(), None),
            quit: MenuItem::new("", true, None),
        };
        items.set_labels(lang);
        items
    }

    fn set_labels(&self, lang: Lang) {
        self.open.set_text(text(lang, Text::OpenSettings));
        self.copy.set_text(text(lang, Text::CopyUrl));
        self.startup.set_text(text(lang, Text::StartWithWindows));
        self.quit.set_text(text(lang, Text::Quit));
    }
}

pub fn run(state: Arc<AppState>) {
    let port = state.port;
    let widget_url = format!("http://localhost:{port}/");
    let config_url = format!("http://localhost:{port}/config");

    let items = Items::new(state.config().lang);
    let menu = Menu::new();
    menu.append_items(&[
        &items.open,
        &items.copy,
        &PredefinedMenuItem::separator(),
        &items.startup,
        &PredefinedMenuItem::separator(),
        &items.quit,
    ])
    .expect("failed to build tray menu");

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(format!("pinguplay · localhost:{port}"))
        .with_icon(icon())
        .build()
        .expect("failed to create tray icon");

    // Os itens do menu não são Send: quem muda o idioma só acorda esta thread.
    // SAFETY: apenas lê o id da thread atual.
    let main_thread = unsafe { GetCurrentThreadId() };
    state.on_lang_change(move || {
        // SAFETY: posta uma mensagem sem ponteiros na fila da thread principal.
        let _ = unsafe { PostThreadMessageW(main_thread, WM_APP_LANG_CHANGED, WPARAM(0), LPARAM(0)) };
    });

    let menu_events = MenuEvent::receiver();
    let mut msg = MSG::default();
    // SAFETY: loop de mensagens padrão do Win32 na thread que criou o ícone.
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 > 0 {
        if msg.message == WM_APP_LANG_CHANGED {
            items.set_labels(state.config().lang);
            continue;
        }
        // SAFETY: `msg` veio de GetMessageW nesta mesma thread.
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        while let Ok(event) = menu_events.try_recv() {
            if event.id == *items.open.id() {
                open_in_browser(&config_url);
            } else if event.id == *items.copy.id() {
                let _ = clipboard_win::set_clipboard_string(&widget_url);
            } else if event.id == *items.startup.id() {
                // O CheckMenuItem já alternou o visto; se o registro falhar, volta ao estado real.
                if autostart::set_enabled(items.startup.is_checked()).is_err() {
                    items.startup.set_checked(autostart::is_enabled());
                }
            } else if event.id == *items.quit.id() {
                return;
            }
        }
    }
}

pub fn open_in_browser(url: &str) {
    // SAFETY: ShellExecuteW com strings válidas; o resultado é ignorado de propósito.
    unsafe {
        ShellExecuteW(None, w!("open"), &HSTRING::from(url), None, None, SW_SHOWNORMAL);
    }
}

/// Ícone gerado em código (disco rosa com furo, tipo vinil) para não depender de arquivo.
fn icon() -> Icon {
    let size = ICON_SIZE as f32;
    let center = (size - 1.0) / 2.0;
    let mut rgba = Vec::with_capacity((ICON_SIZE * ICON_SIZE * 4) as usize);
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let d = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            let pixel = if d > size / 2.0 - 0.5 {
                [0, 0, 0, 0]
            } else if d > size / 2.0 - 3.0 {
                [0xc9, 0x3b, 0x78, 0xff]
            } else if d > 4.0 {
                [0xff, 0x6f, 0xa5, 0xff]
            } else {
                [0xff, 0xf0, 0xf6, 0xff]
            };
            rgba.extend_from_slice(&pixel);
        }
    }
    Icon::from_rgba(rgba, ICON_SIZE, ICON_SIZE).expect("valid icon")
}
