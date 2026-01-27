//! L3: parse the base block of a monitor's EDID (Extended Display Identification Data).
//!
//! Layout from VESA E-EDID Standard Release A2 (EDID 1.4), section 3:
//! - bytes 0–7: header `00 FF FF FF FF FF FF 00`; byte 127: checksum (the 128 bytes sum to 0)
//! - bytes 8–9: manufacturer id, three 5-bit letters (`A` = 1), big-endian
//! - bytes 10–11: product code, little-endian
//! - bytes 21–22: maximum image size in centimetres (0 = unknown or variable)
//! - bytes 54–125: four 18-byte descriptors. A detailed timing descriptor (pixel clock ≠ 0)
//!   carries the active size in pixels and the image size in millimetres; a display
//!   descriptor with tag `FC` carries the monitor name (up to 13 bytes, ended by `0A`).
//!
//! Only the base block is read; extension blocks are ignored. Input is untrusted: the length
//! is capped and every field is checked.

/// Longest EDID accepted: the base block plus 255 extension blocks.
pub const MAX_LEN: usize = 128 * 256;
const BLOCK: usize = 128;
const HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EdidError {
    #[error("EDID is {0} bytes; a base block needs 128")]
    TooShort(usize),
    #[error("EDID is {0} bytes; more than {MAX_LEN}")]
    TooLong(usize),
    #[error("not an EDID: wrong header")]
    BadHeader,
    #[error("EDID checksum mismatch")]
    BadChecksum,
}

/// What pixelprobe uses from an EDID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edid {
    /// Three-letter PNP manufacturer id, e.g. `DEL`, `APP`.
    pub manufacturer: String,
    pub product_code: u16,
    /// Monitor name from the `FC` descriptor.
    pub name: Option<String>,
    /// Image size in millimetres (0 when unknown): from the first detailed timing when it
    /// has one, else from the centimetre fields.
    pub width_mm: u32,
    pub height_mm: u32,
    /// Active pixels of the first detailed timing (the preferred mode).
    pub preferred: Option<(u32, u32)>,
}

fn byte(b: &[u8], i: usize) -> u32 {
    b.get(i).copied().map_or(0, u32::from)
}

pub fn parse(bytes: &[u8]) -> Result<Edid, EdidError> {
    if bytes.len() > MAX_LEN {
        return Err(EdidError::TooLong(bytes.len()));
    }
    let base = bytes.get(..BLOCK).ok_or(EdidError::TooShort(bytes.len()))?;
    if base.get(..8) != Some(&HEADER[..]) {
        return Err(EdidError::BadHeader);
    }
    if base.iter().fold(0u8, |s, b| s.wrapping_add(*b)) != 0 {
        return Err(EdidError::BadChecksum);
    }

    let id = (byte(base, 8) << 8) | byte(base, 9);
    let manufacturer = [10, 5, 0]
        .iter()
        .map(|shift| {
            let v = (id >> shift) & 0x1F;
            match u8::try_from(v) {
                Ok(v @ 1..=26) => char::from(b'A' + v - 1),
                _ => '?',
            }
        })
        .collect();
    let product_code = u16::try_from(byte(base, 10) | (byte(base, 11) << 8)).unwrap_or(0);

    let mut name = None;
    let mut timing = None;
    for start in [54, 72, 90, 108] {
        let Some(d) = base.get(start..start + 18) else {
            continue;
        };
        let clock = byte(d, 0) | (byte(d, 1) << 8);
        if clock != 0 {
            if timing.is_none() {
                timing = Some(detailed_timing(d));
            }
        } else if byte(d, 3) == 0xFC && name.is_none() {
            name = descriptor_text(d);
        }
    }

    let (mut width_mm, mut height_mm) = (byte(base, 21) * 10, byte(base, 22) * 10);
    let mut preferred = None;
    if let Some((w, h, wmm, hmm)) = timing {
        preferred = (w > 0 && h > 0).then_some((w, h));
        if wmm > 0 && hmm > 0 {
            (width_mm, height_mm) = (wmm, hmm);
        }
    }
    Ok(Edid {
        manufacturer,
        product_code,
        name,
        width_mm,
        height_mm,
        preferred,
    })
}

/// (active width, active height, image width mm, image height mm) of an 18-byte detailed
/// timing descriptor (EDID 1.4 table 3.21).
fn detailed_timing(d: &[u8]) -> (u32, u32, u32, u32) {
    let w = byte(d, 2) | ((byte(d, 4) & 0xF0) << 4);
    let h = byte(d, 5) | ((byte(d, 7) & 0xF0) << 4);
    let wmm = byte(d, 12) | ((byte(d, 14) & 0xF0) << 4);
    let hmm = byte(d, 13) | ((byte(d, 14) & 0x0F) << 8);
    (w, h, wmm, hmm)
}

/// The text of a display descriptor (bytes 5–17), up to the `0A` terminator, printable
/// ASCII only, trimmed.
fn descriptor_text(d: &[u8]) -> Option<String> {
    let text: String = d
        .get(5..18)?
        .iter()
        .take_while(|b| **b != 0x0A)
        .filter(|b| b.is_ascii_graphic() || **b == b' ')
        .map(|b| char::from(*b))
        .collect();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

pub mod fill;
pub use fill::{Source, fill};

#[cfg(test)]
pub(crate) mod tests;
