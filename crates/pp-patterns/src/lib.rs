//! L2: pure CPU generation of full-screen test patterns and pixel refresh noise.
//!
//! Patterns are rendered at the display's physical resolution so that pixel-level patterns
//! (1-px checkerboard, grid lines) map 1:1 onto device pixels. Output is full-range 8-bit
//! sRGB (what a PC display expects): a signal level of `p` % becomes `round(p × 255 / 100)`.

mod motion;
mod noise;
mod smpte;

pub use motion::motion_x;
pub use noise::{NOISE_COLOURS, noise};

pub use pp_core::{PatternId, Rgb8};

/// Largest accepted side, in pixels.
pub const MAX_SIDE: u32 = 16_384;
/// Largest accepted pixel count (an 8K UHD display: 7680 × 4320).
pub const MAX_PIXELS: u64 = 7680 * 4320;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatternError {
    #[error("pattern size {width}×{height} is empty or too large")]
    BadSize { width: u32, height: u32 },
}

/// An RGBA8 image, row-major, top row first, alpha always 255.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Image {
    fn new(width: u32, height: u32, fill: Rgb8) -> Result<Self, PatternError> {
        let pixels = u64::from(width) * u64::from(height);
        if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE || pixels > MAX_PIXELS
        {
            return Err(PatternError::BadSize { width, height });
        }
        let n = usize::try_from(pixels).map_err(|_| PatternError::BadSize { width, height })?;
        let mut rgba = Vec::with_capacity(n.saturating_mul(4));
        for _ in 0..n {
            rgba.extend_from_slice(&[fill.r, fill.g, fill.b, 255]);
        }
        Ok(Self {
            width,
            height,
            rgba,
        })
    }

    /// The colour at (x, y), or `None` outside the image.
    pub fn pixel(&self, x: u32, y: u32) -> Option<Rgb8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = (y as usize * self.width as usize + x as usize) * 4;
        match self.rgba.get(i..i + 3) {
            Some([r, g, b]) => Some(Rgb8::new(*r, *g, *b)),
            _ => None,
        }
    }

    /// Fill `[x0, x1) × [y0, y1)`, clipped to the image.
    fn fill(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, c: Rgb8) {
        self.fill_with(x0, y0, x1, y1, |_, _| c);
    }

    /// Fill `[x0, x1) × [y0, y1)` (clipped) with `f(x, y)`.
    fn fill_with(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, f: impl Fn(u32, u32) -> Rgb8) {
        let (x1, y1) = (x1.min(self.width), y1.min(self.height));
        let w = self.width as usize;
        for y in y0..y1 {
            let start = (y as usize * w + x0 as usize) * 4;
            let end = (y as usize * w + x1 as usize) * 4;
            let Some(row) = self.rgba.get_mut(start..end) else {
                continue;
            };
            for (px, x) in row.chunks_exact_mut(4).zip(x0..x1) {
                let c = f(x, y);
                px.copy_from_slice(&[c.r, c.g, c.b, 255]);
            }
        }
    }
}

/// `p` percent of full scale as an 8-bit full-range value (clamped to 0…255).
pub fn level(percent: f64) -> u8 {
    let v = (percent * 255.0 / 100.0).round();
    if v.is_finite() {
        v.clamp(0.0, 255.0) as u8
    } else {
        0
    }
}

/// Gray at `percent`.
pub fn gray(percent: f64) -> Rgb8 {
    let v = level(percent);
    Rgb8::new(v, v, v)
}

/// Pixel position of fraction `f` along a side of `len` pixels.
fn at(f: f64, len: u32) -> u32 {
    let v = (f * f64::from(len)).round();
    if v.is_finite() {
        v.clamp(0.0, f64::from(len)) as u32
    } else {
        0
    }
}

/// Linear 0…255 ramp value for position `i` of `n`.
fn ramp(i: u32, n: u32) -> u8 {
    if n <= 1 {
        return 0;
    }
    let v = u64::from(i.min(n - 1)) * 255 / u64::from(n - 1);
    u8::try_from(v).unwrap_or(u8::MAX)
}

/// Render `pattern` at `width × height` physical pixels.
pub fn render(pattern: PatternId, width: u32, height: u32) -> Result<Image, PatternError> {
    let solid = |c| Image::new(width, height, c);
    let (w, h) = (width, height);
    match pattern {
        PatternId::SmpteBars => smpte::render(w, h),
        PatternId::White => solid(Rgb8::WHITE),
        PatternId::Black => solid(Rgb8::BLACK),
        PatternId::Red => solid(Rgb8::new(255, 0, 0)),
        PatternId::Green => solid(Rgb8::new(0, 255, 0)),
        PatternId::Blue => solid(Rgb8::new(0, 0, 255)),
        PatternId::Gray50 => solid(gray(50.0)),
        PatternId::GrayRamp => {
            let mut img = solid(Rgb8::BLACK)?;
            img.fill_with(0, 0, w, h, |x, _| {
                let v = ramp(x, w);
                Rgb8::new(v, v, v)
            });
            Ok(img)
        }
        PatternId::GraySteps => {
            let mut img = solid(Rgb8::BLACK)?;
            for i in 0..=10u32 {
                let (x0, x1) = (at(f64::from(i) / 11.0, w), at(f64::from(i + 1) / 11.0, w));
                img.fill(x0, 0, x1, h, gray(f64::from(i) * 10.0));
            }
            Ok(img)
        }
        PatternId::RgbRamps => {
            let mut img = solid(Rgb8::BLACK)?;
            let bands: [fn(u8) -> Rgb8; 3] = [
                |v| Rgb8::new(v, 0, 0),
                |v| Rgb8::new(0, v, 0),
                |v| Rgb8::new(0, 0, v),
            ];
            for (i, band) in (0u32..).zip(bands) {
                let (y0, y1) = (at(f64::from(i) / 3.0, h), at(f64::from(i + 1) / 3.0, h));
                img.fill_with(0, y0, w, y1, |x, _| band(ramp(x, w)));
            }
            Ok(img)
        }
        PatternId::BlackLevel => patches(w, h, Rgb8::BLACK, |i| 2 * i),
        PatternId::WhiteLevel => patches(w, h, Rgb8::WHITE, |i| 235 + 2 * i),
        PatternId::Grid => {
            let mut img = solid(Rgb8::BLACK)?;
            img.fill_with(0, 0, w, h, |x, y| {
                let line = x % 64 == 0 || y % 64 == 0 || x + 1 == w || y + 1 == h;
                if line { Rgb8::WHITE } else { Rgb8::BLACK }
            });
            Ok(img)
        }
        PatternId::CheckerFine => checker(w, h, 1),
        PatternId::CheckerCoarse => checker(w, h, 64),
    }
}

fn checker(w: u32, h: u32, cell: u32) -> Result<Image, PatternError> {
    let mut img = Image::new(w, h, Rgb8::BLACK)?;
    let cell = cell.max(1);
    img.fill_with(0, 0, w, h, |x, y| {
        if (x / cell + y / cell).is_multiple_of(2) {
            Rgb8::WHITE
        } else {
            Rgb8::BLACK
        }
    });
    Ok(img)
}

/// 11 square patches across the middle, patch `i` at 8-bit level `value(i)`.
fn patches(w: u32, h: u32, bg: Rgb8, value: fn(u8) -> u8) -> Result<Image, PatternError> {
    let mut img = Image::new(w, h, bg)?;
    let (y0, y1) = (at(0.4, h), at(0.6, h));
    for i in 0..11u8 {
        let x0 = at((f64::from(i) + 0.15) / 11.0, w);
        let x1 = at((f64::from(i) + 0.85) / 11.0, w);
        let v = value(i);
        img.fill(x0, y0, x1, y1, Rgb8::new(v, v, v));
    }
    Ok(img)
}

#[cfg(test)]
mod tests;
