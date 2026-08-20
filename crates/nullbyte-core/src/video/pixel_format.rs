//! Pikselių formatų konversija į RGBA8 (CLAUDE.md §8.4, P2.1).
//!
//! Core paduoda `pitch` **baitais**, ne pikseliais — eilutės gali turėti padding'ą, todėl
//! kiekvienos eilutės pradžia skaičiuojama `y * pitch`, o ne `y * width * bpp`. Tai
//! dažniausia FFI vaizdo konvertavimo klaida (CLAUDE.md §10 „Spąstai").
//!
//! Visi trys formatai — native endian; kadangi Nullbyte taikosi tik į little-endian
//! platformas (x86_64 / aarch64, macOS + Linux — CLAUDE.md §11.5), native endian == LE.

// Naudos video::frame_buffer (P2.2) ir core::callbacks (pixel_format laukas) — kol P2.2
// neparašytas, šis modulis pilnai išnaudojamas tik testuose.
#![allow(dead_code)]

/// Šaltinio pikselių formatas, kurį core praneša per `RETRO_ENVIRONMENT_SET_PIXEL_FORMAT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// `0RGB1555` — 15 bitų, MSB nenaudojamas.
    Rgb0555,
    /// `XRGB8888` — 32 bitų, MSB baitas (X) ignoruojamas.
    Xrgb8888,
    /// `RGB565` — 16 bitų, rekomenduojamas libretro formatas.
    Rgb565,
}

impl PixelFormat {
    /// Baitai vienam pikseliui šaltinio formate.
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            PixelFormat::Rgb0555 => 2,
            PixelFormat::Xrgb8888 => 4,
            PixelFormat::Rgb565 => 2,
        }
    }
}

/// Konvertuoja vieną kadrą į RGBA8, alokuodamas naują `Vec`. Karštame kelyje (per kadrą,
/// pvz. P2.2 triple buffer) naudok [`convert_to_rgba8_into`], kad išvengtum pasikartojančio
/// alokavimo.
pub fn convert_to_rgba8(
    src: &[u8],
    format: PixelFormat,
    width: u32,
    height: u32,
    pitch: usize,
) -> Vec<u8> {
    let mut dst = vec![0u8; width as usize * height as usize * 4];
    convert_to_rgba8_into(src, format, width, height, pitch, &mut dst);
    dst
}

/// Kaip [`convert_to_rgba8`], bet rašo į jau alokuotą `dst` buferį (dydis turi būti bent
/// `width * height * 4` baitų) — vengia alokacijos kiekvienam kadrui.
///
/// # Panic
/// Panikuoja (per `debug_assert!` / slice indeksavimą), jei `src` per trumpas nurodytam
/// `width`/`height`/`pitch`, arba `dst` per mažas — tai programavimo klaida, ne vykdymo laiko
/// situacija, tad nėra prasmės grąžinti `Result`.
pub fn convert_to_rgba8_into(
    src: &[u8],
    format: PixelFormat,
    width: u32,
    height: u32,
    pitch: usize,
    dst: &mut [u8],
) {
    let width = width as usize;
    let height = height as usize;
    debug_assert!(
        dst.len() >= width * height * 4,
        "dst buferis per mažas: {} < {}",
        dst.len(),
        width * height * 4
    );

    match format {
        PixelFormat::Rgb565 => convert_rows(src, width, height, pitch, dst, 2, rgb565_to_rgba8),
        PixelFormat::Xrgb8888 => convert_rows(src, width, height, pitch, dst, 4, xrgb8888_to_rgba8),
        PixelFormat::Rgb0555 => convert_rows(src, width, height, pitch, dst, 2, rgb0555_to_rgba8),
    }
}

#[inline]
fn convert_rows(
    src: &[u8],
    width: usize,
    height: usize,
    pitch: usize,
    dst: &mut [u8],
    src_bpp: usize,
    pixel_fn: impl Fn(&[u8]) -> [u8; 4],
) {
    for y in 0..height {
        let row_start = y * pitch; // PITCH baitais, ne width * src_bpp — žr. modulio doc.
        let row = &src[row_start..row_start + width * src_bpp];
        let dst_row = &mut dst[y * width * 4..(y + 1) * width * 4];
        for x in 0..width {
            let px = pixel_fn(&row[x * src_bpp..x * src_bpp + src_bpp]);
            dst_row[x * 4..x * 4 + 4].copy_from_slice(&px);
        }
    }
}

#[inline]
fn rgb565_to_rgba8(px: &[u8]) -> [u8; 4] {
    let value = u16::from_le_bytes([px[0], px[1]]);
    let r5 = (value >> 11) & 0x1F;
    let g6 = (value >> 5) & 0x3F;
    let b5 = value & 0x1F;
    [scale5(r5), scale6(g6), scale5(b5), 255]
}

#[inline]
fn rgb0555_to_rgba8(px: &[u8]) -> [u8; 4] {
    let value = u16::from_le_bytes([px[0], px[1]]);
    let r5 = (value >> 10) & 0x1F;
    let g5 = (value >> 5) & 0x1F;
    let b5 = value & 0x1F;
    [scale5(r5), scale5(g5), scale5(b5), 255]
}

#[inline]
fn xrgb8888_to_rgba8(px: &[u8]) -> [u8; 4] {
    // native-endian (LE) XRGB8888 baitų tvarka atmintyje: [B, G, R, X] — MSB baitas (X)
    // ignoruojamas.
    [px[2], px[1], px[0], 255]
}

/// 5 bitų (0–31) → 8 bitų (0–255) bit-replikacijos metodu — tikslesnis nei paprastas `*8`,
/// nes pilnas 5 bitų `1` laukas (31) tampa lygiai 255, ne 248.
#[inline]
fn scale5(v: u16) -> u8 {
    ((v << 3) | (v >> 2)) as u8
}

/// 6 bitų (0–63) → 8 bitų (0–255), ta pati logika kaip [`scale5`].
#[inline]
fn scale6(v: u16) -> u8 {
    ((v << 2) | (v >> 4)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb565_pixel(r5: u16, g6: u16, b5: u16) -> [u8; 2] {
        let value = (r5 << 11) | (g6 << 5) | b5;
        value.to_le_bytes()
    }

    fn rgb0555_pixel(r5: u16, g5: u16, b5: u16) -> [u8; 2] {
        let value = (r5 << 10) | (g5 << 5) | b5;
        value.to_le_bytes()
    }

    #[test]
    fn rgb565_known_colors() {
        assert_eq!(
            rgb565_to_rgba8(&rgb565_pixel(31, 63, 31)),
            [255, 255, 255, 255]
        );
        assert_eq!(rgb565_to_rgba8(&rgb565_pixel(0, 0, 0)), [0, 0, 0, 255]);
        assert_eq!(rgb565_to_rgba8(&rgb565_pixel(31, 0, 0)), [255, 0, 0, 255]);
        assert_eq!(rgb565_to_rgba8(&rgb565_pixel(0, 63, 0)), [0, 255, 0, 255]);
        assert_eq!(rgb565_to_rgba8(&rgb565_pixel(0, 0, 31)), [0, 0, 255, 255]);
    }

    #[test]
    fn rgb0555_known_colors() {
        assert_eq!(
            rgb0555_to_rgba8(&rgb0555_pixel(31, 31, 31)),
            [255, 255, 255, 255]
        );
        assert_eq!(rgb0555_to_rgba8(&rgb0555_pixel(0, 0, 0)), [0, 0, 0, 255]);
        assert_eq!(rgb0555_to_rgba8(&rgb0555_pixel(31, 0, 0)), [255, 0, 0, 255]);
        assert_eq!(rgb0555_to_rgba8(&rgb0555_pixel(0, 31, 0)), [0, 255, 0, 255]);
        assert_eq!(rgb0555_to_rgba8(&rgb0555_pixel(0, 0, 31)), [0, 0, 255, 255]);
    }

    #[test]
    fn xrgb8888_known_colors() {
        // native-endian (LE) atmintyje: [B, G, R, X].
        assert_eq!(
            xrgb8888_to_rgba8(&[0x33, 0x22, 0x11, 0xFF]),
            [0x11, 0x22, 0x33, 255]
        );
        assert_eq!(
            xrgb8888_to_rgba8(&[0xFF, 0xFF, 0xFF, 0x00]),
            [255, 255, 255, 255]
        );
        assert_eq!(xrgb8888_to_rgba8(&[0x00, 0x00, 0x00, 0xFF]), [0, 0, 0, 255]);
    }

    #[test]
    fn convert_rgb565_full_frame_matches_per_pixel() {
        let width = 4u32;
        let height = 2u32;
        let pitch = width as usize * 2;
        let mut src = vec![0u8; pitch * height as usize];
        // (0,0) raudona, (1,0) žalia, likusieji juoda.
        src[0..2].copy_from_slice(&rgb565_pixel(31, 0, 0));
        src[2..4].copy_from_slice(&rgb565_pixel(0, 63, 0));

        let rgba = convert_to_rgba8(&src, PixelFormat::Rgb565, width, height, pitch);

        assert_eq!(rgba.len(), (width * height * 4) as usize);
        assert_eq!(&rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&rgba[4..8], &[0, 255, 0, 255]);
        assert_eq!(&rgba[8..12], &[0, 0, 0, 255]);
    }

    #[test]
    fn respects_pitch_padding_larger_than_row_width() {
        let width = 2u32;
        let height = 2u32;
        let bpp = 2;
        let padding = 6; // papildomi baitai po kiekvienos eilutės, kuriuos reikia praleisti
        let pitch = width as usize * bpp + padding;

        let mut src = vec![0xAAu8; pitch * height as usize]; // "šiukšlės" paddinge
                                                             // 1-a eilutė: 2 raudoni pikseliai.
        src[0..2].copy_from_slice(&rgb565_pixel(31, 0, 0));
        src[2..4].copy_from_slice(&rgb565_pixel(31, 0, 0));
        // 2-a eilutė (po pitch, ne po width*bpp!): 2 mėlyni pikseliai.
        src[pitch..pitch + 2].copy_from_slice(&rgb565_pixel(0, 0, 31));
        src[pitch + 2..pitch + 4].copy_from_slice(&rgb565_pixel(0, 0, 31));

        let rgba = convert_to_rgba8(&src, PixelFormat::Rgb565, width, height, pitch);

        assert_eq!(
            &rgba[0..4],
            &[255, 0, 0, 255],
            "1-a eilutė turėtų būti raudona"
        );
        assert_eq!(&rgba[4..8], &[255, 0, 0, 255]);
        assert_eq!(
            &rgba[8..12],
            &[0, 0, 255, 255],
            "2-a eilutė turėtų būti mėlyna, ne šiukšlės"
        );
        assert_eq!(&rgba[12..16], &[0, 0, 255, 255]);
    }

    #[test]
    fn benchmark_256x224_rgb565_under_half_millisecond() {
        let width = 256u32;
        let height = 224u32;
        let pitch = width as usize * 2;
        let src = vec![0x55u8; pitch * height as usize];
        let mut dst = vec![0u8; width as usize * height as usize * 4];

        // Apšilimas — pirmas kvietimas gali būti lėtesnis (cache/instrukcijų puslapiai).
        for _ in 0..10 {
            convert_to_rgba8_into(&src, PixelFormat::Rgb565, width, height, pitch, &mut dst);
        }

        let iterations = 100;
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            convert_to_rgba8_into(&src, PixelFormat::Rgb565, width, height, pitch, &mut dst);
        }
        let per_call = start.elapsed() / iterations;

        // debug build'e (be optimizacijų) šis testas gali būti gerokai lėtesnis už release —
        // acceptance kriterijus (< 0.5 ms) realiai taikomas release build'ui.
        let limit_ms = if cfg!(debug_assertions) { 5.0 } else { 0.5 };
        assert!(
            per_call.as_secs_f64() * 1000.0 < limit_ms,
            "konversija per lėta: {:?} (limitas {limit_ms} ms, debug={})",
            per_call,
            cfg!(debug_assertions)
        );
    }
}
