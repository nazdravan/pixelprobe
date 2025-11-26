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

/// The whole user-editable state of a test session ("the document").
///
/// Small and `Clone`: the engine snapshots it for undo and for panic rollback.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Doc {
    pub screen: Screen,
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
    fn doc_starts_on_main_screen() {
        assert_eq!(Doc::default().screen, Screen::Main);
    }
}
