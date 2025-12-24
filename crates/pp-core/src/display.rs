//! Display (monitor) facts and the values derived from them. Pure data: the app fills it from
//! the platform; everything here is computed, never queried.

use serde::{Deserialize, Serialize};

/// Most displays and modes accepted from the platform (bounds untrusted input).
pub const MAX_DISPLAYS: usize = 32;
pub const MAX_MODES: usize = 1024;
/// Longest display name kept, in bytes.
pub const MAX_NAME: usize = 256;

/// One display mode the display supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    /// Bits per pixel; 0 when unknown.
    pub bits: u16,
    /// Refresh rate in millihertz; 0 when unknown.
    pub refresh_mhz: u32,
}

impl DisplayMode {
    pub fn refresh_hz(&self) -> Option<f64> {
        (self.refresh_mhz > 0).then(|| f64::from(self.refresh_mhz) / 1000.0)
    }
}

/// What the platform reports about one display.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayInfo {
    pub name: String,
    pub primary: bool,
    pub builtin: bool,
    /// The app window is on this display.
    pub has_window: bool,
    /// Top-left corner on the desktop, in physical pixels.
    pub x: i32,
    pub y: i32,
    /// Current resolution in physical pixels.
    pub width: u32,
    pub height: u32,
    /// Physical pixels per desktop point (2.0 on a Retina display).
    pub scale: f64,
    /// Current refresh rate in millihertz; 0 when unknown.
    pub refresh_mhz: u32,
    /// Bits per pixel of the current mode; 0 when unknown.
    pub bits: u16,
    /// Visible image size in millimetres; 0 when unknown.
    pub width_mm: f64,
    pub height_mm: f64,
    /// Clockwise rotation in degrees.
    pub rotation: f64,
    /// Supported modes, sorted, without duplicates.
    pub modes: Vec<DisplayMode>,
}

/// Greatest common divisor (0 only for (0, 0)).
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

const MM_PER_INCH: f64 = 25.4;

impl DisplayInfo {
    /// Clamp everything the platform sent to sane, finite values and bounded sizes.
    pub fn sanitized(mut self) -> Self {
        if self.name.len() > MAX_NAME {
            let mut end = MAX_NAME;
            while !self.name.is_char_boundary(end) {
                end -= 1;
            }
            self.name.truncate(end);
        }
        // A visible image 1 cm to 10 m wide is plausible; anything else counts as unknown.
        let finite_pos = |v: f64| {
            if (10.0..=10_000.0).contains(&v) {
                v
            } else {
                0.0
            }
        };
        // Real scale factors are 1–4; anything outside 0.25–16 is garbage.
        self.scale = if (0.25..=16.0).contains(&self.scale) {
            self.scale
        } else {
            1.0
        };
        self.width_mm = finite_pos(self.width_mm);
        self.height_mm = finite_pos(self.height_mm);
        self.rotation = if self.rotation.is_finite() {
            self.rotation
        } else {
            0.0
        };
        self.modes.sort_unstable();
        self.modes.dedup();
        self.modes.truncate(MAX_MODES);
        self
    }

    pub fn refresh_hz(&self) -> Option<f64> {
        (self.refresh_mhz > 0).then(|| f64::from(self.refresh_mhz) / 1000.0)
    }

    /// Desktop size in points (what apps lay out in).
    pub fn desktop_size(&self) -> (f64, f64) {
        (
            f64::from(self.width) / self.scale,
            f64::from(self.height) / self.scale,
        )
    }

    fn has_size_mm(&self) -> bool {
        self.width_mm > 0.0 && self.height_mm > 0.0
    }

    /// Pixels per inch horizontally and vertically, when the physical size is known.
    pub fn dpi(&self) -> Option<(f64, f64)> {
        self.has_size_mm().then(|| {
            (
                f64::from(self.width) * MM_PER_INCH / self.width_mm,
                f64::from(self.height) * MM_PER_INCH / self.height_mm,
            )
        })
    }

    /// Diagonal of the visible image in inches, when the physical size is known.
    pub fn diagonal_inches(&self) -> Option<f64> {
        self.has_size_mm()
            .then(|| self.width_mm.hypot(self.height_mm) / MM_PER_INCH)
    }

    /// Diagonal in pixels.
    pub fn diagonal_pixels(&self) -> f64 {
        f64::from(self.width).hypot(f64::from(self.height))
    }

    /// Width : height of the resolution, reduced (e.g. 16:9, 16:10, 64:27).
    pub fn screen_aspect(&self) -> Option<(u32, u32)> {
        let g = gcd(self.width, self.height);
        (g > 0 && self.width > 0 && self.height > 0).then(|| (self.width / g, self.height / g))
    }

    /// Width of one pixel over its height (1.0 = square pixels), from the physical size.
    pub fn pixel_aspect(&self) -> Option<f64> {
        let (x, y) = self.dpi()?;
        (x > 0.0).then(|| y / x)
    }

    /// Number of distinct colours for the current bit depth (24 → 16 777 216). Alpha and
    /// padding bits do not add colours, so 32 bpp counts as 24.
    pub fn colours(&self) -> Option<u64> {
        let colour_bits = match self.bits {
            0 => return None,
            32 => 24,
            b => b.min(48),
        };
        Some(1u64 << colour_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn studio_display() -> DisplayInfo {
        // 27" 5K: 5120 × 2880 px, 596.7 × 335.7 mm.
        DisplayInfo {
            name: "Studio".into(),
            width: 5120,
            height: 2880,
            scale: 2.0,
            width_mm: 596.7,
            height_mm: 335.7,
            refresh_mhz: 60_000,
            bits: 32,
            ..Default::default()
        }
    }

    #[test]
    fn derived_values() {
        let d = studio_display();
        let (dx, dy) = d.dpi().unwrap();
        assert!((dx - 217.9).abs() < 0.1, "{dx}");
        assert!((dy - 217.9).abs() < 0.1, "{dy}");
        assert!((d.diagonal_inches().unwrap() - 26.95).abs() < 0.01);
        assert_eq!(d.screen_aspect(), Some((16, 9)));
        assert!((d.pixel_aspect().unwrap() - 1.0).abs() < 0.01);
        assert_eq!(d.desktop_size(), (2560.0, 1440.0));
        assert_eq!(d.colours(), Some(16_777_216));
        assert!((d.diagonal_pixels() - 5874.4).abs() < 0.1);
        assert_eq!(d.refresh_hz(), Some(60.0));
    }

    #[test]
    fn unknowns_stay_unknown() {
        let d = DisplayInfo {
            width: 1920,
            height: 1080,
            ..Default::default()
        }
        .sanitized();
        assert_eq!(d.dpi(), None);
        assert_eq!(d.diagonal_inches(), None);
        assert_eq!(d.pixel_aspect(), None);
        assert_eq!(d.colours(), None);
        assert_eq!(d.refresh_hz(), None);
        assert_eq!(d.scale, 1.0);
        assert_eq!(DisplayInfo::default().screen_aspect(), None);
        assert_eq!(
            DisplayInfo {
                width: 3440,
                height: 1440,
                ..Default::default()
            }
            .screen_aspect(),
            Some((43, 18))
        );
    }

    #[test]
    fn sanitize_bounds_bad_input() {
        let d = DisplayInfo {
            name: "é".repeat(200),
            scale: f64::NAN,
            width_mm: -3.0,
            height_mm: f64::INFINITY,
            rotation: f64::NAN,
            modes: vec![
                DisplayMode {
                    width: 1,
                    height: 1,
                    bits: 0,
                    refresh_mhz: 0
                };
                MAX_MODES + 10
            ],
            ..Default::default()
        }
        .sanitized();
        assert!(d.name.len() <= MAX_NAME);
        assert_eq!(
            (d.scale, d.width_mm, d.height_mm, d.rotation),
            (1.0, 0.0, 0.0, 0.0)
        );
        assert_eq!(d.modes.len(), 1, "duplicates removed");
    }

    #[test]
    fn colour_counts() {
        let with_bits = |bits| DisplayInfo {
            bits,
            ..Default::default()
        };
        assert_eq!(with_bits(16).colours(), Some(65_536));
        assert_eq!(with_bits(30).colours(), Some(1 << 30));
        assert_eq!(with_bits(64).colours(), Some(1 << 48));
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn derived_values_are_finite_for_any_input(
            width: u32, height: u32, bits: u16, refresh_mhz: u32,
            scale in proptest::num::f64::ANY, w_mm in proptest::num::f64::ANY,
            h_mm in proptest::num::f64::ANY, name in ".{0,300}",
        ) {
            let d = DisplayInfo {
                name, width, height, bits, refresh_mhz, scale,
                width_mm: w_mm, height_mm: h_mm, ..Default::default()
            }.sanitized();
            prop_assert!(d.name.len() <= MAX_NAME);
            let (dw, dh) = d.desktop_size();
            prop_assert!(dw.is_finite() && dh.is_finite());
            prop_assert!(d.diagonal_pixels().is_finite());
            if let Some((x, y)) = d.dpi() {
                prop_assert!(x.is_finite() && y.is_finite());
            }
            if let Some((a, b)) = d.screen_aspect() {
                prop_assert!(a > 0 && b > 0);
            }
            if let Some(a) = d.pixel_aspect() {
                prop_assert!(a.is_finite());
            }
            if let Some(i) = d.diagonal_inches() {
                prop_assert!(i.is_finite());
            }
            let _ = (d.colours(), d.refresh_hz());
        }
    }
}
