//! Pixel refresh noise: every cell takes one of red, green, blue, white or black at random.
//!
//! Deterministic: frame `seed` always gives the same image, so tests and screenshots repeat.

use crate::{Image, PatternError, Rgb8};

/// The five colours a cell can take: each subpixel is driven fully on and fully off.
pub const NOISE_COLOURS: [Rgb8; 5] = [
    Rgb8::new(255, 0, 0),
    Rgb8::new(0, 255, 0),
    Rgb8::new(0, 0, 255),
    Rgb8::WHITE,
    Rgb8::BLACK,
];

/// SplitMix64 (Steele, Lea, Flood 2014): a fast, well-mixed hash of a 64-bit counter.
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A `cols × rows` image, one pixel per cell, for noise frame `seed`.
pub fn noise(cols: u32, rows: u32, seed: u64) -> Result<Image, PatternError> {
    let mut img = Image::new(cols, rows, Rgb8::BLACK)?;
    let base = splitmix64(seed);
    img.fill_with(0, 0, cols, rows, |x, y| {
        let cell = u64::from(y) * u64::from(cols) + u64::from(x);
        let r = splitmix64(base ^ cell.wrapping_mul(0xD6E8_FEB8_6659_FD93));
        let i = usize::try_from(r % 5).unwrap_or(0);
        NOISE_COLOURS.get(i).copied().unwrap_or(Rgb8::BLACK)
    });
    Ok(img)
}
