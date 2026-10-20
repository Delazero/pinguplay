//! Arquivos de animação (GIF/WebP/APNG) que aparecem no lado esquerdo do card.

use std::{fs, path::Path};

pub const DEFAULT_GIF: &[u8] = include_bytes!("../assets/penguin.gif");
pub const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Gif,
    Png,
    Webp,
}

impl Format {
    pub fn ext(self) -> &'static str {
        match self {
            Format::Gif => "gif",
            Format::Png => "png",
            Format::Webp => "webp",
        }
    }

    pub fn mime(self) -> &'static str {
        match self {
            Format::Gif => "image/gif",
            Format::Png => "image/png",
            Format::Webp => "image/webp",
        }
    }

    pub fn from_ext(ext: &str) -> Option<Format> {
        match ext.to_ascii_lowercase().as_str() {
            "gif" => Some(Format::Gif),
            "png" | "apng" => Some(Format::Png),
            "webp" => Some(Format::Webp),
            _ => None,
        }
    }

    pub fn from_mime(mime: &str) -> Option<Format> {
        match mime {
            "image/gif" => Some(Format::Gif),
            "image/png" | "image/apng" => Some(Format::Png),
            "image/webp" => Some(Format::Webp),
            _ => None,
        }
    }
}

/// Detecta o formato pelos bytes iniciais, sem confiar na extensão.
pub fn sniff(bytes: &[u8]) -> Option<Format> {
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some(Format::Gif)
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(Format::Png)
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(Format::Webp)
    } else {
        None
    }
}

/// Nome simples de arquivo dentro de `assets/`: sem pastas, sem `..`, extensão conhecida.
pub fn is_safe_name(name: &str) -> bool {
    let Some((stem, ext)) = name.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty()
        && !stem.starts_with('.')
        && !is_reserved(stem)
        && name.len() <= 100
        && stem.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' '))
        && Format::from_ext(ext).is_some()
}

/// Gera um nome seguro para um upload, mantendo o nome original quando possível.
pub fn safe_file_name(original: &str, format: Format) -> String {
    let stem = original.rsplit_once('.').map_or(original, |(stem, _)| stem);
    let mut clean: String = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') { c } else { '_' })
        .take(60)
        .collect();
    clean = clean.trim_matches('_').to_string();
    if clean.is_empty() {
        clean = "animation".into();
    }
    if is_reserved(&clean) {
        clean.insert(0, '_');
    }
    format!("{clean}.{}", format.ext())
}

/// Nomes de dispositivo do Windows (CON, NUL, COM1…) não podem virar arquivo.
fn is_reserved(stem: &str) -> bool {
    let base = stem.split('.').next().unwrap_or(stem).trim_end().to_ascii_uppercase();
    matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((base.starts_with("COM") || base.starts_with("LPT"))
            && base.len() == 4
            && base.as_bytes()[3].is_ascii_digit())
}

pub fn format_of_name(name: &str) -> Option<Format> {
    name.rsplit_once('.').and_then(|(_, ext)| Format::from_ext(ext))
}

/// Animações disponíveis na pasta `assets/`, em ordem alfabética.
pub fn list(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| is_safe_name(name))
        .collect();
    names.sort_by_key(|n| n.to_lowercase());
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_formats() {
        assert_eq!(sniff(b"GIF89a...."), Some(Format::Gif));
        assert_eq!(sniff(b"GIF87a...."), Some(Format::Gif));
        assert_eq!(sniff(b"\x89PNG\r\n\x1a\nrest"), Some(Format::Png));
        assert_eq!(sniff(b"RIFF\0\0\0\0WEBPVP8 "), Some(Format::Webp));
        assert_eq!(sniff(b"MZ\x90\0"), None);
        assert_eq!(sniff(b""), None);
    }

    #[test]
    fn default_penguin_is_a_gif() {
        assert_eq!(sniff(DEFAULT_GIF), Some(Format::Gif));
    }

    #[test]
    fn safe_names() {
        assert!(is_safe_name("penguin.gif"));
        assert!(is_safe_name("my cat-2.webp"));
        assert!(is_safe_name("dance.PNG"));
        assert!(!is_safe_name("../x.gif"));
        assert!(!is_safe_name("a/b.gif"));
        assert!(!is_safe_name("a\\b.gif"));
        assert!(!is_safe_name(".gif"));
        assert!(!is_safe_name("evil.exe"));
        assert!(!is_safe_name("noext"));
        assert!(!is_safe_name("CON.gif"));
        assert!(!is_safe_name("com1.png"));
        assert!(is_safe_name("console.gif"));
    }

    #[test]
    fn reserved_upload_names_are_prefixed() {
        assert_eq!(safe_file_name("nul.gif", Format::Gif), "_nul.gif");
        assert_eq!(safe_file_name("LPT9.webp", Format::Webp), "_LPT9.webp");
    }

    #[test]
    fn upload_names_are_cleaned() {
        assert_eq!(safe_file_name("Pinguim Dança!.gif", Format::Gif), "Pinguim_Dan_a.gif");
        assert_eq!(safe_file_name("../../x.exe", Format::Png), "x.png");
        assert_eq!(safe_file_name("???", Format::Webp), "animation.webp");
        assert!(is_safe_name(&safe_file_name("C:\\a\\b.gif", Format::Gif)));
    }

    #[test]
    fn lists_only_animation_files() {
        let dir = std::env::temp_dir().join(format!("pinguplay-anim-{}", std::process::id()));
        fs::create_dir_all(dir.join("sub.gif")).unwrap();
        for name in ["b.gif", "A.webp", "notes.txt"] {
            fs::write(dir.join(name), b"x").unwrap();
        }
        assert_eq!(list(&dir), vec!["A.webp", "b.gif"]);
        fs::remove_dir_all(&dir).unwrap();
    }
}
