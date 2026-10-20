use crate::config::Lang;

#[derive(Debug, Clone, Copy)]
pub enum Text {
    OpenSettings,
    CopyUrl,
    StartWithWindows,
    Quit,
    /// Contém `{port}`.
    PortBusy,
}

pub fn text(lang: Lang, key: Text) -> &'static str {
    match (lang, key) {
        (Lang::Pt, Text::OpenSettings) => "Abrir configurações",
        (Lang::En, Text::OpenSettings) => "Open settings",
        (Lang::Pt, Text::CopyUrl) => "Copiar URL do widget",
        (Lang::En, Text::CopyUrl) => "Copy widget URL",
        (Lang::Pt, Text::StartWithWindows) => "Iniciar com o Windows",
        (Lang::En, Text::StartWithWindows) => "Start with Windows",
        (Lang::Pt, Text::Quit) => "Sair",
        (Lang::En, Text::Quit) => "Quit",
        (Lang::Pt, Text::PortBusy) => {
            "A porta {port} já está em uso.\n\nO pinguplay já está aberto? Veja o ícone na bandeja. \
             Se não, troque a porta no config.toml."
        }
        (Lang::En, Text::PortBusy) => {
            "Port {port} is already in use.\n\nIs pinguplay already running? Check the tray icon. \
             If not, change the port in config.toml."
        }
    }
}
