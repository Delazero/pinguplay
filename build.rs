// Embute ícone e metadados (nome do produto, versão, descrição) no pinguplay.exe.
// A SignPath exige nome e versão do produto consistentes nos binários assinados.

use std::{env, fs, path::PathBuf};

include!("src/icon.rs");

const ICON_SIZES: [u32; 5] = [16, 24, 32, 48, 64];

fn main() {
    println!("cargo:rerun-if-changed=src/icon.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let ico = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("pinguplay.ico");
    fs::write(&ico, ico_bytes(&ICON_SIZES)).expect("failed to write icon");

    // FileVersion/ProductVersion saem do `version` do Cargo.toml.
    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico.to_str().expect("utf-8 path"))
        .set("ProductName", "pinguplay")
        .set("FileDescription", "pinguplay - now playing widget for OBS")
        .set("LegalCopyright", "Copyright (c) 2026 pinguplay contributors. MIT License.");
    res.compile().expect("failed to compile Windows resources");
}

/// Arquivo .ico com uma entrada BMP de 32 bits por tamanho.
fn ico_bytes(sizes: &[u32]) -> Vec<u8> {
    let images: Vec<Vec<u8>> = sizes.iter().map(|&size| bmp_entry(size)).collect();
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]);
    out.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (&size, image) in sizes.iter().zip(&images) {
        out.extend_from_slice(&[size as u8, size as u8, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes()); // planos
        out.extend_from_slice(&32u16.to_le_bytes()); // bits por pixel
        out.extend_from_slice(&(image.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += image.len() as u32;
    }
    images.into_iter().for_each(|image| out.extend(image));
    out
}

/// DIB 32 bits BGRA de baixo pra cima, seguido da máscara AND (zerada: o alfa manda).
fn bmp_entry(size: u32) -> Vec<u8> {
    let rgba = disc_rgba(size);
    let mask_row = size.div_ceil(32) * 4;
    let mut out = Vec::new();
    out.extend_from_slice(&40u32.to_le_bytes()); // BITMAPINFOHEADER
    out.extend_from_slice(&(size as i32).to_le_bytes());
    out.extend_from_slice(&((size * 2) as i32).to_le_bytes()); // altura conta cor + máscara
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    out.extend_from_slice(&(size * size * 4 + mask_row * size).to_le_bytes());
    out.extend_from_slice(&[0u8; 16]); // resolução e paleta
    for y in (0..size).rev() {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            out.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
        }
    }
    out.extend(std::iter::repeat_n(0u8, (mask_row * size) as usize));
    out
}
