use super::*;
use proptest::prelude::*;

fn px(img: &Image, x: u32, y: u32) -> Rgb8 {
    img.pixel(x, y).unwrap()
}

fn g(v: u8) -> Rgb8 {
    Rgb8::new(v, v, v)
}

#[test]
fn levels() {
    assert_eq!(level(0.0), 0);
    assert_eq!(level(100.0), 255);
    assert_eq!(level(75.0), 191);
    assert_eq!(level(40.0), 102);
    assert_eq!(level(15.0), 38);
    assert_eq!(level(-2.0), 0);
    assert_eq!(level(2.0), 5);
    assert_eq!(level(4.0), 10);
    assert_eq!(level(f64::NAN), 0);
}

#[test]
fn rejects_bad_sizes() {
    for (w, h) in [(0, 10), (10, 0), (MAX_SIDE + 1, 1), (8000, 8000)] {
        assert!(render(PatternId::White, w, h).is_err(), "{w}x{h}");
    }
}

#[test]
fn smpte_rp219_at_1920x1080() {
    let img = render(PatternId::SmpteBars, 1920, 1080).unwrap();
    // d = 240, c = 1440 / 7 ≈ 205.7; pattern 1 is rows 0..630.
    assert_eq!(px(&img, 100, 300), g(102), "40% gray side panel");
    assert_eq!(px(&img, 1820, 300), g(102));
    let bars = [
        Rgb8::new(191, 191, 191),
        Rgb8::new(191, 191, 0),
        Rgb8::new(0, 191, 191),
        Rgb8::new(0, 191, 0),
        Rgb8::new(191, 0, 191),
        Rgb8::new(191, 0, 0),
        Rgb8::new(0, 0, 191),
    ];
    for (i, want) in (0u32..).zip(bars) {
        let x = 240 + i * 1440 / 7 + 100;
        assert_eq!(px(&img, x, 300), want, "bar {i}");
    }
    // Pattern 2 (rows 630..720): 100% cyan, 75% white, 100% blue.
    assert_eq!(px(&img, 100, 675), Rgb8::new(0, 255, 255));
    assert_eq!(px(&img, 960, 675), g(191));
    assert_eq!(px(&img, 1820, 675), Rgb8::new(0, 0, 255));
    // Pattern 3 (rows 720..810): yellow, black, ramp, red.
    assert_eq!(px(&img, 100, 765), Rgb8::new(255, 255, 0));
    assert_eq!(px(&img, 300, 765), g(0));
    assert_eq!(px(&img, 1820, 765), Rgb8::new(255, 0, 0));
    let ramp: Vec<u8> = (446..1680).map(|x| px(&img, x, 765).r).collect();
    assert!(ramp.windows(2).all(|p| p[0] <= p[1]), "ramp is monotonic");
    assert_eq!(ramp.first(), Some(&0));
    assert_eq!(ramp.last(), Some(&255));
    // Pattern 4 (rows 810..1080): 15% gray, black, white 2c, black, PLUGE, black, 15% gray.
    assert_eq!(px(&img, 100, 1000), g(38));
    assert_eq!(px(&img, 400, 1000), g(0));
    assert_eq!(px(&img, 700, 1000), g(255));
    // PLUGE steps start at d + (3/2 + 2 + 5/6) c = 240 + 4.333 c ≈ 1131.4, each c/3 ≈ 68.6.
    let step = |i: f64| (240.0 + (1440.0 / 7.0) * (13.0 / 3.0 + i / 3.0) + 30.0) as u32;
    assert_eq!(px(&img, step(0.0), 1000), g(0), "-2% clips to black");
    assert_eq!(px(&img, step(2.0), 1000), g(5), "+2%");
    assert_eq!(px(&img, step(4.0), 1000), g(10), "+4%");
    assert_eq!(px(&img, 1820, 1000), g(38));
}

#[test]
fn smpte_has_no_gaps_at_odd_sizes() {
    // Every pixel is painted by some segment: no pixel keeps an unexpected colour.
    let img = render(PatternId::SmpteBars, 1366, 768).unwrap();
    assert_eq!(img.rgba.len(), 1366 * 768 * 4);
    assert_eq!(px(&img, 0, 0), g(102));
    assert_eq!(px(&img, 1365, 0), g(102));
    assert_eq!(px(&img, 1365, 767), g(38));
}

#[test]
fn pixel_patterns() {
    let c = render(PatternId::CheckerFine, 8, 4).unwrap();
    assert_eq!(px(&c, 0, 0), Rgb8::WHITE);
    assert_eq!(px(&c, 1, 0), Rgb8::BLACK);
    assert_eq!(px(&c, 1, 1), Rgb8::WHITE);
    let c = render(PatternId::CheckerCoarse, 200, 200).unwrap();
    assert_eq!(px(&c, 63, 0), Rgb8::WHITE);
    assert_eq!(px(&c, 64, 0), Rgb8::BLACK);
    let grid = render(PatternId::Grid, 130, 70).unwrap();
    assert_eq!(px(&grid, 0, 10), Rgb8::WHITE);
    assert_eq!(px(&grid, 64, 10), Rgb8::WHITE);
    assert_eq!(px(&grid, 65, 10), Rgb8::BLACK);
    assert_eq!(px(&grid, 129, 10), Rgb8::WHITE, "right border");
    assert_eq!(px(&grid, 10, 69), Rgb8::WHITE, "bottom border");
}

#[test]
fn ramps_and_steps() {
    let r = render(PatternId::GrayRamp, 256, 4).unwrap();
    assert_eq!(px(&r, 0, 0), g(0));
    assert_eq!(px(&r, 255, 0), g(255));
    assert_eq!(px(&r, 128, 0), g(128));
    let s = render(PatternId::GraySteps, 1100, 10).unwrap();
    assert_eq!(px(&s, 50, 5), g(0));
    assert_eq!(px(&s, 550, 5), g(128), "50% step");
    assert_eq!(px(&s, 1099, 5), g(255));
    let rgb = render(PatternId::RgbRamps, 256, 300).unwrap();
    assert_eq!(px(&rgb, 255, 50), Rgb8::new(255, 0, 0));
    assert_eq!(px(&rgb, 255, 150), Rgb8::new(0, 255, 0));
    assert_eq!(px(&rgb, 255, 250), Rgb8::new(0, 0, 255));
}

#[test]
fn level_patches() {
    let b = render(PatternId::BlackLevel, 1100, 100).unwrap();
    assert_eq!(px(&b, 50, 50), g(0));
    assert_eq!(px(&b, 1050, 50), g(20));
    assert_eq!(px(&b, 1050, 10), g(0), "background");
    let wl = render(PatternId::WhiteLevel, 1100, 100).unwrap();
    assert_eq!(px(&wl, 50, 50), g(235));
    assert_eq!(px(&wl, 1050, 10), g(255));
}

proptest! {
    #[test]
    fn any_pattern_any_size_is_complete(
        idx in 0..PatternId::ALL.len(),
        w in 1u32..96,
        h in 1u32..96,
    ) {
        let p = PatternId::ALL[idx];
        let img = render(p, w, h).unwrap();
        prop_assert_eq!(img.rgba.len(), (w * h * 4) as usize);
        prop_assert!(img.rgba.chunks_exact(4).all(|px| px[3] == 255));
    }

    #[test]
    fn any_size_never_panics(w in any::<u32>(), h in any::<u32>()) {
        // Only cheap sizes get rendered; the rest must be rejected without allocating.
        if u64::from(w) * u64::from(h) > 4096 {
            prop_assume!(u64::from(w) * u64::from(h) > MAX_PIXELS || w > MAX_SIDE || h > MAX_SIDE);
        }
        let _ = render(PatternId::SmpteBars, w, h);
    }
}

#[test]
fn noise_uses_only_the_five_colours_evenly() {
    let img = noise(100, 100, 42).unwrap();
    let mut counts = [0u32; 5];
    for y in 0..100 {
        for x in 0..100 {
            let c = px(&img, x, y);
            let i = NOISE_COLOURS.iter().position(|n| *n == c).unwrap();
            counts[i] += 1;
        }
    }
    // 10 000 cells, 2 000 expected per colour.
    for n in counts {
        assert!((1700..2300).contains(&n), "{counts:?}");
    }
}

#[test]
fn noise_is_deterministic_and_changes_per_frame() {
    assert_eq!(noise(40, 30, 7).unwrap(), noise(40, 30, 7).unwrap());
    let (a, b) = (noise(40, 30, 7).unwrap(), noise(40, 30, 8).unwrap());
    let changed = (0..30)
        .flat_map(|y| (0..40).map(move |x| (x, y)))
        .filter(|&(x, y)| px(&a, x, y) != px(&b, x, y))
        .count();
    // A cell keeps its colour with probability 1/5: ~960 of 1200 change.
    assert!(changed > 800, "{changed}");
    assert!(noise(0, 10, 1).is_err());
    assert!(noise(MAX_SIDE + 1, 1, 1).is_err());
}

proptest! {
    #[test]
    fn noise_any_size_and_seed(cols in 0u32..300, rows in 0u32..300, seed: u64) {
        match noise(cols, rows, seed) {
            Ok(img) => {
                prop_assert!(cols > 0 && rows > 0);
                prop_assert_eq!(img.rgba.len(), cols as usize * rows as usize * 4);
            }
            Err(_) => prop_assert!(cols == 0 || rows == 0),
        }
    }
}

#[test]
fn smpte_sub_patterns_rp219_clause_2() {
    // At 1920×1080: d = 240, c ≈ 205.7. *2 is x 240..446 in rows 630..720, *3 below it.
    let at2 = |v| {
        px(
            &render_with(PatternId::SmpteBars, v, 1920, 1080).unwrap(),
            300,
            675,
        )
    };
    let at3 = |v| {
        px(
            &render_with(PatternId::SmpteBars, v, 1920, 1080).unwrap(),
            300,
            765,
        )
    };
    assert_eq!(at2(SmpteVariant::White75), g(191));
    assert_eq!(at2(SmpteVariant::White100), g(255));
    // +I: R 41.2545 → 105, G 16.6946 → 43, B 0.
    assert_eq!(at2(SmpteVariant::PlusI), Rgb8::new(105, 43, 0));
    // −I: G 24.56 → 63, B 41.2545 → 105; +Q: R 25.3605 → 65, B 47.0286 → 120.
    assert_eq!(at2(SmpteVariant::MinusIPlusQ), Rgb8::new(0, 63, 105));
    assert_eq!(at3(SmpteVariant::MinusIPlusQ), Rgb8::new(65, 0, 120));
    for v in [
        SmpteVariant::White75,
        SmpteVariant::White100,
        SmpteVariant::PlusI,
    ] {
        assert_eq!(at3(v), g(0), "{v:?}: *3 stays black");
    }
    // The default renderer is the 75 % white variant.
    assert_eq!(
        render(PatternId::SmpteBars, 640, 360).unwrap(),
        render_with(PatternId::SmpteBars, SmpteVariant::White75, 640, 360).unwrap()
    );
}
