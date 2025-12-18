//! L0–L1: colour, geometry and the test-session model. Plain data, no I/O.

use serde::{Deserialize, Serialize};

/// An sRGB colour with 8 bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb8 {
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Which screen of the app is in front.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Screen {
    #[default]
    Main,
    MotionTest,
    PatternTest,
    PixelRefresh,
    About,
}

/// A full-screen test pattern. Order here is the order of prev/next navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternId {
    /// SMPTE RP 219:2002 HD/SD-compatible colour bars.
    #[default]
    SmpteBars,
    White,
    Black,
    Red,
    Green,
    Blue,
    Gray50,
    /// Horizontal luminance ramp 0 → 100 %.
    GrayRamp,
    /// 11 vertical steps 0 %, 10 %, … 100 %.
    GraySteps,
    /// Red, green and blue ramps stacked.
    RgbRamps,
    /// Near-black patches 0…20 of 255 on black.
    BlackLevel,
    /// Near-white patches 235…255 of 255 on white.
    WhiteLevel,
    /// 1-pixel white grid lines every 64 pixels, with a 1-pixel border.
    Grid,
    /// 1×1-pixel black/white checkerboard.
    CheckerFine,
    /// 64×64-pixel black/white checkerboard.
    CheckerCoarse,
}

impl PatternId {
    pub const ALL: [Self; 15] = [
        Self::SmpteBars,
        Self::White,
        Self::Black,
        Self::Red,
        Self::Green,
        Self::Blue,
        Self::Gray50,
        Self::GrayRamp,
        Self::GraySteps,
        Self::RgbRamps,
        Self::BlackLevel,
        Self::WhiteLevel,
        Self::Grid,
        Self::CheckerFine,
        Self::CheckerCoarse,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::SmpteBars => "SMPTE bars",
            Self::White => "White",
            Self::Black => "Black",
            Self::Red => "Red",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Gray50 => "Gray 50%",
            Self::GrayRamp => "Gray ramp",
            Self::GraySteps => "Gray steps",
            Self::RgbRamps => "RGB ramps",
            Self::BlackLevel => "Black level",
            Self::WhiteLevel => "White level",
            Self::Grid => "Grid",
            Self::CheckerFine => "Checkerboard 1 px",
            Self::CheckerCoarse => "Checkerboard 64 px",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|p| *p == self).unwrap_or(0)
    }

    /// The next pattern, wrapping around.
    pub fn next(self) -> Self {
        let i = (self.index() + 1) % Self::ALL.len();
        Self::ALL.get(i).copied().unwrap_or_default()
    }

    /// The previous pattern, wrapping around.
    pub fn prev(self) -> Self {
        let n = Self::ALL.len();
        let i = (self.index() + n - 1) % n;
        Self::ALL.get(i).copied().unwrap_or_default()
    }
}

/// Pixel refresh speed: a slider position 1…10, mapped to a noise update rate.
///
/// Out-of-range values are rejected when deserialized, so a `RefreshSpeed` is always valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct RefreshSpeed(u8);

impl RefreshSpeed {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 10;
    /// Noise updates per second for each speed, slowest first. 60 /s matches the most common
    /// display refresh rate; faster updates would only drop frames.
    const RATES: [u32; 10] = [1, 2, 5, 10, 15, 20, 30, 40, 50, 60];

    pub fn new(speed: u8) -> Result<Self, String> {
        if (Self::MIN..=Self::MAX).contains(&speed) {
            Ok(Self(speed))
        } else {
            Err(format!(
                "speed must be {}…{}, got {speed}",
                Self::MIN,
                Self::MAX
            ))
        }
    }

    pub fn get(self) -> u8 {
        self.0
    }

    /// Noise updates per second.
    pub fn rate_hz(self) -> u32 {
        let i = usize::from(self.0.saturating_sub(Self::MIN));
        Self::RATES.get(i).copied().unwrap_or(1)
    }
}

impl Default for RefreshSpeed {
    /// 30 updates per second.
    fn default() -> Self {
        Self(7)
    }
}

impl TryFrom<u8> for RefreshSpeed {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, String> {
        Self::new(v)
    }
}

impl From<RefreshSpeed> for u8 {
    fn from(s: RefreshSpeed) -> u8 {
        s.0
    }
}

/// The whole user-editable state of a test session ("the document").
///
/// Small and `Clone`: the engine snapshots it for undo and for panic rollback.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Doc {
    pub screen: Screen,
    /// Pattern shown by the pattern test.
    pub pattern: PatternId,
    /// Noise update speed of the pixel refresh.
    #[serde(default)]
    pub refresh_speed: RefreshSpeed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants() {
        assert_eq!(Rgb8::WHITE, Rgb8::new(255, 255, 255));
        assert_ne!(Rgb8::BLACK, Rgb8::WHITE);
    }

    #[test]
    fn pattern_navigation_wraps() {
        assert_eq!(PatternId::SmpteBars.prev(), PatternId::CheckerCoarse);
        assert_eq!(PatternId::CheckerCoarse.next(), PatternId::SmpteBars);
        let mut p = PatternId::default();
        for _ in 0..PatternId::ALL.len() {
            p = p.next();
        }
        assert_eq!(p, PatternId::default());
        for p in PatternId::ALL {
            assert_eq!(p.next().prev(), p);
            assert!(!p.name().is_empty());
        }
    }

    #[test]
    fn refresh_speed_range_and_rates() {
        assert_eq!(RefreshSpeed::default().rate_hz(), 30);
        assert!(RefreshSpeed::new(0).is_err());
        assert!(RefreshSpeed::new(11).is_err());
        let mut last = 0;
        for v in RefreshSpeed::MIN..=RefreshSpeed::MAX {
            let rate = RefreshSpeed::new(v).unwrap().rate_hz();
            assert!(rate > last, "rates increase");
            last = rate;
        }
        assert_eq!(last, 60);
        let s: RefreshSpeed = serde_json::from_str("3").unwrap();
        assert_eq!(s.get(), 3);
        assert!(serde_json::from_str::<RefreshSpeed>("0").is_err());
        assert!(serde_json::from_str::<RefreshSpeed>("300").is_err());
    }

    #[test]
    fn doc_starts_on_main_screen() {
        assert_eq!(Doc::default().screen, Screen::Main);
    }
}
