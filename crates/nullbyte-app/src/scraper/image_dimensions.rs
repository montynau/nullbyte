//! Minimalus paveikslėlio matmenų (plotis/aukštis) nuskaitymas iš PNG/JPEG baitų — TIK
//! header'io perskaitymas, be dekodavimo, kad nereikėtų sunkios `image` crate priklausomybės
//! vien šiai smulkiai reikmei (ADR-021, MVP.md P7.2 GameGrid „packed row" layout'ui — tikri
//! viršelio matmenys reikalingi, nes ScreenScraper box-2D proporcijos LABAI skiriasi tarp
//! platformų, žr. `006_game_cover_dimensions.sql`).

/// Bando PNG, tada JPEG. `None`, jei nei vienas formatas neatpažintas arba baitai per trumpi.
pub fn read_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    read_png_dimensions(data).or_else(|| read_jpeg_dimensions(data))
}

/// PNG: 8 baitų signatūra, tada IHDR chunk'as visada PIRMAS ir VISADA 13 baitų — plotis
/// (4 baitai BE) iškart po `length`+`"IHDR"` (8 baitai), aukštis — dar 4 baitai toliau.
fn read_png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data.len() < 24 || data[..8] != SIGNATURE {
        return None;
    }
    let width = u32::from_be_bytes(data[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(data[20..24].try_into().ok()?);
    Some((width, height))
}

/// JPEG: skenuoja markerius nuo `0xFFD8` (SOI), praleisdamas kiekvieno segmento duomenis
/// pagal jo `length` lauką, kol randa SOF (Start Of Frame) markerį — ten laikomi aukštis/
/// plotis. Standalone markeriai (`0xD0`-`0xD9`, `0x01`) neturi `length` lauko.
fn read_jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut pos = 2;
    while pos + 1 < data.len() {
        if data[pos] != 0xFF {
            pos += 1;
            continue;
        }
        let marker = data[pos + 1];
        if marker == 0x01 || (0xD0..=0xD9).contains(&marker) {
            pos += 2;
            continue;
        }
        if pos + 4 > data.len() {
            return None;
        }
        let segment_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        let is_sof = matches!(
            marker,
            0xC0 | 0xC1
                | 0xC2
                | 0xC3
                | 0xC5
                | 0xC6
                | 0xC7
                | 0xC9
                | 0xCA
                | 0xCB
                | 0xCD
                | 0xCE
                | 0xCF
        );
        if is_sof {
            if pos + 9 > data.len() {
                return None;
            }
            let height = u16::from_be_bytes([data[pos + 5], data[pos + 6]]) as u32;
            let width = u16::from_be_bytes([data[pos + 7], data[pos + 8]]) as u32;
            return Some((width, height));
        }
        pos += 2 + segment_len;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_png(width: u32, height: u32) -> Vec<u8> {
        let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        data.extend_from_slice(&[0, 0, 0, 13]); // IHDR chunk length
        data.extend_from_slice(b"IHDR");
        data.extend_from_slice(&width.to_be_bytes());
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&[0; 5]); // likusi IHDR dalis — nesvarbi šiam testui
        data
    }

    #[test]
    fn reads_real_png_dimensions() {
        assert_eq!(read_dimensions(&make_png(680, 497)), Some((680, 497)));
        assert_eq!(read_dimensions(&make_png(680, 680)), Some((680, 680)));
    }

    fn make_minimal_jpeg(width: u16, height: u16) -> Vec<u8> {
        let mut data = vec![0xFF, 0xD8]; // SOI
                                         // APP0 segmentas (turi būti praleistas per `segment_len`).
        data.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x04, 0xAB, 0xCD]);
        // SOF0 segmentas: length=8 (2 baitai) + precision(1) + height(2) + width(2) + components(1).
        data.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x08, 0x08]);
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&width.to_be_bytes());
        data.push(0x03);
        data
    }

    #[test]
    fn reads_real_jpeg_dimensions_skipping_earlier_segments() {
        assert_eq!(
            read_dimensions(&make_minimal_jpeg(705, 700)),
            Some((705, 700))
        );
    }

    #[test]
    fn rejects_garbage_and_too_short_input() {
        assert_eq!(read_dimensions(b"not an image"), None);
        assert_eq!(read_dimensions(b""), None);
        assert_eq!(read_dimensions(&[0xFF, 0xD8]), None);
    }
}
