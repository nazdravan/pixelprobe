//! SMPTE RP 219:2002 "High-Definition, Standard-Definition Compatible Color Bar Signal".
//! Source: https://pub.smpte.org/pub/rp219/Rp219-2002.pdf (figure 1 and clause 2).
//!
//! Geometry (figure 1), for picture width `a` and height `b` (a:b = 16:9):
//! - band heights: pattern 1 = 7/12 b, pattern 2 = 1/12 b, pattern 3 = 1/12 b, pattern 4 = 3/12 b
//! - the 4:3 area is 3/4 a wide and holds 7 bars of width `c` = (3/4 a) / 7
//! - side panels are `d` = (a − 7c) / 2 = a / 8 wide
//!
//! Levels are percentages of full scale (IRE units map 1:1). *1 = 40 % gray, *4 = 15 % gray;
//! *2 and *3 follow the chosen [`SmpteVariant`]: 75 % white (default), 100 % white,
//! +I (R 41.2545, G 16.6946, B 0) or −I (R 0, G 24.56, B 41.2545) with +Q
//! (R 25.3605, G 0, B 47.0286) in *3 (clause 2 and notes 4–5).
//! On a full-range display the −2 % PLUGE step clips to black, which is the intended result:
//! it must be indistinguishable from 0 % when black level is set correctly.
//!
//! On non-16:9 displays the same fractions of the width are used (the pattern is stretched).

use crate::{Image, PatternError, Rgb8, SmpteVariant, at, gray, level, ramp};

fn rgb(r: f64, g: f64, b: f64) -> Rgb8 {
    Rgb8::new(level(r), level(g), level(b))
}

pub fn render(variant: SmpteVariant, w: u32, h: u32) -> Result<Image, PatternError> {
    let mut img = Image::new(w, h, Rgb8::BLACK)?;
    let d = 1.0 / 8.0;
    let c = (3.0 / 4.0) / 7.0;
    let rows = [0.0, 7.0 / 12.0, 8.0 / 12.0, 9.0 / 12.0, 1.0].map(|f| at(f, h));
    let [y0, y1, y2, y3, y4] = rows;

    // Lay out `segments` (fraction-of-width, colour) left to right in band [top, bottom).
    let band = |img: &mut Image, top: u32, bottom: u32, segments: &[(f64, Rgb8)]| {
        let mut x = 0.0;
        for &(width, colour) in segments {
            img.fill(at(x, w), top, at(x + width, w), bottom, colour);
            x += width;
        }
    };

    // Pattern 1: 40 % gray, 75 % bars W Y C G M R B, 40 % gray.
    let g40 = gray(40.0);
    band(
        &mut img,
        y0,
        y1,
        &[
            (d, g40),
            (c, rgb(75.0, 75.0, 75.0)),
            (c, rgb(75.0, 75.0, 0.0)),
            (c, rgb(0.0, 75.0, 75.0)),
            (c, rgb(0.0, 75.0, 0.0)),
            (c, rgb(75.0, 0.0, 75.0)),
            (c, rgb(75.0, 0.0, 0.0)),
            (c, rgb(0.0, 0.0, 75.0)),
            (d, g40),
        ],
    );

    // *2 and *3 sub-patterns.
    let (star2, star3) = match variant {
        SmpteVariant::White75 => (gray(75.0), gray(0.0)),
        SmpteVariant::White100 => (gray(100.0), gray(0.0)),
        SmpteVariant::PlusI => (rgb(41.2545, 16.6946, 0.0), gray(0.0)),
        SmpteVariant::MinusIPlusQ => (rgb(0.0, 24.56, 41.2545), rgb(25.3605, 0.0, 47.0286)),
    };

    // Pattern 2: 100 % cyan, *2, 75 % white chroma setting signal, 100 % blue.
    band(
        &mut img,
        y1,
        y2,
        &[
            (d, rgb(0.0, 100.0, 100.0)),
            (c, star2),
            (6.0 * c, gray(75.0)),
            (d, rgb(0.0, 0.0, 100.0)),
        ],
    );

    // Pattern 3: 100 % yellow, *3, luminance ramp 0 → 100 %, 100 % red.
    band(
        &mut img,
        y2,
        y3,
        &[
            (d, rgb(100.0, 100.0, 0.0)),
            (c, star3),
            (6.0 * c, gray(0.0)),
            (d, rgb(100.0, 0.0, 0.0)),
        ],
    );
    let (r0, r1) = (at(d + c, w), at(d + 7.0 * c, w));
    let n = r1.saturating_sub(r0);
    img.fill_with(r0, y2, r1, y3, |x, _| {
        let v = ramp(x - r0, n);
        Rgb8::new(v, v, v)
    });

    // Pattern 4: 15 % gray, 0 % black 3/2 c, 100 % white 2c, 0 % black 5/6 c,
    // PLUGE −2 / 0 / +2 / 0 / +4 % at 1/3 c each, 0 % black c, 15 % gray.
    let k = gray(0.0);
    band(
        &mut img,
        y3,
        y4,
        &[
            (d, gray(15.0)),
            (1.5 * c, k),
            (2.0 * c, gray(100.0)),
            (5.0 / 6.0 * c, k),
            (c / 3.0, gray(-2.0)),
            (c / 3.0, k),
            (c / 3.0, gray(2.0)),
            (c / 3.0, k),
            (c / 3.0, gray(4.0)),
            (c, k),
            (d, gray(15.0)),
        ],
    );
    Ok(img)
}
