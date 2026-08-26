//! Minimalus PNG encoder'is — TIK tiek, kiek reikia P8.1 save state preview paveiksliukams
//! (RGBA8 kadras iš `video::frame_buffer::VideoFrameData` → PNG baitai). SĄMONINGAI
//! NENAUDOJA jokios PNG/DEFLATE crate'o produkciniame kode (ta pati filosofija kaip
//! `scraper::image_dimensions`, ADR-021) — naudoja DEFLATE „stored" (nesuspaustus) blokus,
//! tad išvestis DIDESNĖ nei suspaustas PNG, bet preview paveiksliukai maži (SNES
//! 256×224 ≈ 230KB nesuspausta), tad tai nesvarbu. CRC32 — per JAU esančią `crc32fast`
//! priklausomybę (naudojamą ROM hash'ams).

use crc32fast::Hasher as Crc32;

const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
/// zlib srauto antraštė (RFC 1950): CMF=0x78 → compression method=8 (deflate), CINFO=7
/// (32K langas); FLG=0x01 → FLEVEL=0/FDICT=0, FCHECK parinktas taip, kad
/// `(CMF*256 + FLG) % 31 == 0`, kaip reikalauja RFC — abu baitai FIKSUOTI, nes visada
/// naudojame tą patį compression method/window/level.
const ZLIB_HEADER: [u8; 2] = [0x78, 0x01];
const MAX_STORED_BLOCK: usize = 65535;

/// Užkoduoja RGBA8 kadrą (`width*height*4` baitų, be eilučių paddingo) į PNG failo baitus.
/// `None`, jei `width`/`height` == 0 arba `rgba.len()` nesutampa su tikėtinu dydžiu —
/// kviečiančioji pusė (P8.1 `core::savestate`) tokiu atveju tiesiog praleidžia preview'ą,
/// tai NĖRA kritiška klaida pačiam save state'ui.
pub fn encode_rgba8(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    if width == 0 || height == 0 {
        return None;
    }
    let expected_len = width as usize * height as usize * 4;
    if rgba.len() != expected_len {
        return None;
    }

    let mut out = Vec::with_capacity(expected_len + 4096);
    out.extend_from_slice(&PNG_SIGNATURE);
    write_chunk(&mut out, b"IHDR", &ihdr_data(width, height));
    write_chunk(
        &mut out,
        b"IDAT",
        &zlib_stored(&filtered_scanlines(width, height, rgba)),
    );
    write_chunk(&mut out, b"IEND", &[]);
    Some(out)
}

fn ihdr_data(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity(13);
    data.extend_from_slice(&width.to_be_bytes());
    data.extend_from_slice(&height.to_be_bytes());
    data.push(8); // bit depth
    data.push(6); // color type: RGBA (truecolor + alpha)
    data.push(0); // compression method — visada 0 (deflate)
    data.push(0); // filter method — visada 0
    data.push(0); // interlace — jokio
    data
}

/// Kiekvienai eilutei priklijuoja filtro tipo baitą (0 = None — jokio filtravimo, paprasčiausia
/// TEISINGA PNG reikšmė; ne mažiausia, bet mums svarbu paprastumas, ne dydis).
fn filtered_scanlines(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let row_bytes = width as usize * 4;
    let mut out = Vec::with_capacity((row_bytes + 1) * height as usize);
    for row in 0..height as usize {
        out.push(0);
        out.extend_from_slice(&rgba[row * row_bytes..(row + 1) * row_bytes]);
    }
    out
}

/// RFC 1950 zlib srautas su RFC 1951 DEFLATE „stored" (nesuspaustais) blokais — kiekvienas
/// blokas ≤ 65535 baitų (DEFLATE LEN lauko limitas), NLEN = LEN vienetų papildinys.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + data.len() / MAX_STORED_BLOCK.max(1) + 16);
    out.extend_from_slice(&ZLIB_HEADER);

    if data.is_empty() {
        // Bent vienas (tuščias) blokas privalomas net tuščiam srautui.
        out.push(0x01); // BFINAL=1, BTYPE=00 (stored)
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    } else {
        let mut offset = 0;
        while offset < data.len() {
            let chunk_len = (data.len() - offset).min(MAX_STORED_BLOCK);
            let is_last = offset + chunk_len == data.len();
            out.push(if is_last { 0x01 } else { 0x00 });
            let len = chunk_len as u16;
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&(!len).to_le_bytes());
            out.extend_from_slice(&data[offset..offset + chunk_len]);
            offset += chunk_len;
        }
    }

    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

/// RFC 1950 Adler-32 — zlib srauto trailer'is.
fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }
    (b << 16) | a
}

fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let mut crc = Crc32::new();
    crc.update(&out[start..]);
    out.extend_from_slice(&crc.finalize().to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkerboard(width: u32, height: u32) -> Vec<u8> {
        let mut data = vec![0u8; width as usize * height as usize * 4];
        for y in 0..height as usize {
            for x in 0..width as usize {
                let i = (y * width as usize + x) * 4;
                let on = (x + y) % 2 == 0;
                data[i] = if on { 255 } else { 0 };
                data[i + 1] = if on { 0 } else { 255 };
                data[i + 2] = 128;
                data[i + 3] = 255;
            }
        }
        data
    }

    #[test]
    fn rejects_zero_dimensions() {
        assert_eq!(encode_rgba8(0, 10, &[]), None);
        assert_eq!(encode_rgba8(10, 0, &[]), None);
    }

    #[test]
    fn rejects_mismatched_buffer_length() {
        assert_eq!(encode_rgba8(4, 4, &[0u8; 10]), None);
    }

    #[test]
    fn roundtrips_through_a_real_png_decoder() {
        let width = 17; // sąmoningai nelyginis/ne-2^n, kad pagautų off-by-one klaidas
        let height = 13;
        let original = checkerboard(width, height);

        let png_bytes = encode_rgba8(width, height, &original).expect("turėtų užkoduoti");

        let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes.as_slice()));
        let mut reader = decoder
            .read_info()
            .expect("turėtų perskaityti PNG antraštę");
        let info = reader.info();
        assert_eq!(info.width, width);
        assert_eq!(info.height, height);
        assert_eq!(info.color_type, png::ColorType::Rgba);
        assert_eq!(info.bit_depth, png::BitDepth::Eight);

        let mut buf = vec![0u8; reader.output_buffer_size().expect("žinomas dydis")];
        let frame_info = reader.next_frame(&mut buf).expect("turėtų dekoduoti kadrą");
        let decoded = &buf[..frame_info.buffer_size()];

        assert_eq!(decoded, original.as_slice());
    }

    #[test]
    fn roundtrips_a_frame_larger_than_one_stored_block() {
        // 65535 baitų vienas stored blokas — imk kadrą, kurio filtruotos eilutės (su +1
        // baitu per eilutę) VIRŠIJA tai, kad patikrintume kelių blokų sujungimo logiką.
        let width = 300;
        let height = 300; // 300*300*4 + 300 (filter baitai) = 360300 baitų, >> 65535
        let original = checkerboard(width, height);

        let png_bytes = encode_rgba8(width, height, &original).expect("turėtų užkoduoti");

        let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes.as_slice()));
        let mut reader = decoder
            .read_info()
            .expect("turėtų perskaityti PNG antraštę");
        let mut buf = vec![0u8; reader.output_buffer_size().expect("žinomas dydis")];
        let frame_info = reader.next_frame(&mut buf).expect("turėtų dekoduoti kadrą");
        assert_eq!(&buf[..frame_info.buffer_size()], original.as_slice());
    }

    #[test]
    fn adler32_matches_known_test_vector() {
        // "Wikipedia" → 0x11E60398 (paskelbtas pavyzdys Adler-32 aprašyme).
        assert_eq!(adler32(b"Wikipedia"), 0x11E60398);
    }
}
