// Desenho do ícone (disco cinza claro com furo, tipo vinil), compartilhado entre o
// tray (src/tray.rs) e o ícone do .exe (build.rs, via include!). Por isso sem `//!`.

/// Pixels RGBA, linha a linha de cima pra baixo, de um ícone `size`×`size`.
pub fn disc_rgba(size: u32) -> Vec<u8> {
    let s = size as f32;
    let center = (s - 1.0) / 2.0;
    // Proporções do desenho original de 32px: borda de 3px, furo de raio 4px.
    let ring = s * 3.0 / 32.0;
    let hole = s * 4.0 / 32.0;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let d = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            let pixel = if d > s / 2.0 - 0.5 {
                [0, 0, 0, 0]
            } else if d > s / 2.0 - ring {
                [0x71, 0x71, 0x7a, 0xff]
            } else if d > hole {
                [0xe4, 0xe4, 0xe7, 0xff]
            } else {
                [0x18, 0x18, 0x1b, 0xff]
            };
            rgba.extend_from_slice(&pixel);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disc_is_transparent_outside_and_dark_in_the_hole() {
        let size = 32;
        let rgba = disc_rgba(size);
        assert_eq!(rgba.len(), (size * size * 4) as usize);
        let pixel = |x: u32, y: u32| &rgba[((y * size + x) * 4) as usize..][..4];
        assert_eq!(pixel(0, 0), [0, 0, 0, 0]);
        assert_eq!(pixel(16, 16), [0x18, 0x18, 0x1b, 0xff]);
        assert_eq!(pixel(16, 6), [0xe4, 0xe4, 0xe7, 0xff]);
    }
}
