use super::*;
use proptest::prelude::*;

/// A 21.5" 4K iMac panel (serial zeroed). Its base block has no detailed timing (the mode
/// is in a DisplayID extension), so the size comes from the centimetre fields: 48 × 27 cm.
/// CoreGraphics reports 479.4 × 269.7 mm for the same panel.
const IMAC: &[u8] = include_bytes!("../tests/data/imac-21-4k.bin");

#[test]
fn imac_panel() {
    let e = parse(IMAC).unwrap();
    assert_eq!(e.manufacturer, "APP");
    assert_eq!(e.product_code, 0xAE1B);
    assert_eq!(e.name.as_deref(), Some("iMac"));
    assert_eq!((e.width_mm, e.height_mm), (480, 270));
    assert_eq!(e.preferred, None);
}

/// A minimal EDID 1.4 base block: manufacturer DEL, a 2560 × 1440 detailed timing with a
/// 597 × 336 mm image, name "TEST 27", 60 × 34 cm size fields.
pub(crate) fn synthetic() -> Vec<u8> {
    let mut b = vec![0u8; 128];
    b[..8].copy_from_slice(&HEADER);
    // "DEL": D=4, E=5, L=12 → 0 00100 00101 01100.
    let id: u16 = (4 << 10) | (5 << 5) | 12;
    b[8..10].copy_from_slice(&id.to_be_bytes());
    b[10..12].copy_from_slice(&0x1234u16.to_le_bytes());
    b[18] = 1;
    b[19] = 4;
    b[21] = 60;
    b[22] = 34;
    // Detailed timing at 54: clock 241.5 MHz, 2560 × 1440, 597 × 336 mm.
    let d = &mut b[54..72];
    d[0..2].copy_from_slice(&24150u16.to_le_bytes());
    d[2] = (2560 & 0xFF) as u8;
    d[4] = ((2560 >> 8) << 4) as u8;
    d[5] = (1440 & 0xFF) as u8;
    d[7] = ((1440 >> 8) << 4) as u8;
    d[12] = (597 & 0xFF) as u8;
    d[13] = (336 & 0xFF) as u8;
    d[14] = (((597 >> 8) << 4) | (336 >> 8)) as u8;
    // Name descriptor at 72.
    let n = &mut b[72..90];
    n[3] = 0xFC;
    n[5..18].copy_from_slice(b"TEST 27\n     ");
    b[127] = 0u8.wrapping_sub(b[..127].iter().fold(0u8, |s, x| s.wrapping_add(*x)));
    b
}

#[test]
fn detailed_timing_wins_over_centimetres() {
    let e = parse(&synthetic()).unwrap();
    assert_eq!(e.manufacturer, "DEL");
    assert_eq!(e.product_code, 0x1234);
    assert_eq!(e.name.as_deref(), Some("TEST 27"));
    assert_eq!(e.preferred, Some((2560, 1440)));
    assert_eq!((e.width_mm, e.height_mm), (597, 336));
}

#[test]
fn rejects_bad_input() {
    assert_eq!(parse(&[]), Err(EdidError::TooShort(0)));
    assert_eq!(parse(&[0; 127]), Err(EdidError::TooShort(127)));
    assert_eq!(parse(&[0; 128]), Err(EdidError::BadHeader));
    let mut b = synthetic();
    b[100] ^= 1;
    assert_eq!(parse(&b), Err(EdidError::BadChecksum));
    assert_eq!(
        parse(&vec![0; MAX_LEN + 1]),
        Err(EdidError::TooLong(MAX_LEN + 1))
    );
}

proptest! {
    #[test]
    fn never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..600)) {
        let _ = parse(&bytes);
    }

    /// Any valid-header, valid-checksum block parses, with bounded values.
    #[test]
    fn any_checksummed_block_parses(body in proptest::collection::vec(any::<u8>(), 119)) {
        let mut b = HEADER.to_vec();
        b.extend_from_slice(&body);
        b.push(0u8.wrapping_sub(b.iter().fold(0u8, |s, x| s.wrapping_add(*x))));
        let e = parse(&b).unwrap();
        prop_assert_eq!(e.manufacturer.len(), 3);
        prop_assert!(e.width_mm <= 4095 && e.height_mm <= 4095);
        if let Some((w, h)) = e.preferred {
            prop_assert!(w <= 4095 && h <= 4095);
        }
        prop_assert!(e.name.as_ref().is_none_or(|n| n.len() <= 13));
    }
}
