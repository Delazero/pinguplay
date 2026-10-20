//! Ícone na bandeja e loop de mensagens do Windows (roda na thread principal).

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
                DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, SW_SHOWNORMAL, TranslateMessage, WM_QUIT,
            },
        },
    },
    core::{HSTRING, w},
};

use crate::{
    autostart,
    config::Lang,
    i18n::{Text, text},
};

const ICON_SIZE: u32 = 32;

pub fn run(lang: Lang, port: u16) {
    let widget_url = format!("http://localhost:{port}/");
    let config_url = format!("http://localhost:{port}/config");

    let open = MenuItem::new(text(lang, Text::OpenSettings), true, None);
    let copy = MenuItem::new(text(lang, Text::CopyUrl), true, None);
    let startup = CheckMenuItem::new(text(lang, Text::StartWithWindows), true, autostart::is_enabled(), None);
    let quit = MenuItem::new(text(lang, Text::Quit), true, None);
    let menu = Menu::new();
    menu.append_items(&[
        &open,
        &copy,
        &PredefinedMenuItem::separator(),
        &startup,
        &PredefinedMenuItem::separator(),
        &quit,
    ])
    .expect("failed to build tray menu");

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(format!("pinguplay · localhost:{port}"))
        .with_icon(icon())
        .build()
        .expect("failed to create tray icon");

    // SAFETY: apenas lê o id da thread atual.
    let main_thread = unsafe { GetCurrentThreadId() };
    let (open_id, copy_id, startup_id, quit_id) =
        (open.id().clone(), copy.id().clone(), startup.id().clone(), quit.id().clone());
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == open_id {
            open_in_browser(&config_url);
        } else if event.id == copy_id {
            let _ = clipboard_win::set_clipboard_string(&widget_url);
        } else if event.id == startup_id {
            // O CheckMenuItem já alterna o visto sozinho; aqui só espelha no registro.
            let _ = autostart::set_enabled(!autostart::is_enabled());
        } else if event.id == quit_id {
            // SAFETY: posta WM_QUIT na fila da thread principal, que encerra o loop abaixo.
            let _ = unsafe { PostThreadMessageW(main_thread, WM_QUIT, WPARAM(0), LPARAM(0)) };
        }
    }));

    let mut msg = MSG::default();
    // SAFETY: loop de mensagens padrão do Win32 na thread que criou o ícone.
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
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
