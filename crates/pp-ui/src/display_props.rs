//! Text for the monitor list and the display properties table.

use pp_engine::pp_core::DisplayInfo;

const UNKNOWN: &str = "—";

/// One line of the monitor list: name, then what kind of display it is.
pub fn list_label(d: &DisplayInfo) -> String {
    let mut tags = Vec::new();
    if d.primary {
        tags.push("primary");
    }
    if d.builtin {
        tags.push("built-in");
    }
    if d.has_window {
        tags.push("this window");
    }
    let name = if d.name.is_empty() {
        "Display"
    } else {
        &d.name
    };
    let res = format!("{} × {}", d.width, d.height);
    if tags.is_empty() {
        format!("{name}  —  {res}")
    } else {
        format!("{name}  —  {res}  ({})", tags.join(", "))
    }
}

/// `n` with thin thousands separators: 16777216 → "16 777 216".
fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('\u{202F}');
        }
        out.push(c);
    }
    out
}

/// A refresh rate as text: "60 Hz", "59.94 Hz", or — when unknown.
pub fn hz(v: Option<f64>) -> String {
    v.map_or(UNKNOWN.into(), |v| {
        if (v - v.round()).abs() < 0.005 {
            format!("{v:.0} Hz")
        } else {
            format!("{v:.2} Hz")
        }
    })
}

/// The display refresh rate as shown in readouts, e.g. "60 Hz" or "59.94 Hz".
pub fn refresh_text(d: Option<&DisplayInfo>) -> String {
    hz(d.and_then(DisplayInfo::refresh_hz))
}

/// (property, value) rows for the properties table.
pub fn properties(d: &DisplayInfo) -> Vec<(&'static str, String)> {
    let yes_no = |b: bool| if b { "Yes" } else { "No" }.to_owned();
    let (dw, dh) = d.desktop_size();
    let mut rows = vec![
        (
            "Name",
            if d.name.is_empty() {
                UNKNOWN.into()
            } else {
                d.name.clone()
            },
        ),
        ("Primary", yes_no(d.primary)),
        ("Built-in", yes_no(d.builtin)),
        ("Resolution", format!("{} × {} px", d.width, d.height)),
        ("Desktop resolution", format!("{dw:.0} × {dh:.0} pt")),
        ("Scale factor", format!("{:.2}", d.scale)),
        ("Refresh rate", hz(d.refresh_hz())),
        (
            "Bits per pixel",
            if d.bits == 0 {
                UNKNOWN.into()
            } else {
                d.bits.to_string()
            },
        ),
        ("Colours", d.colours().map_or(UNKNOWN.into(), grouped)),
    ];
    let size = if d.width_mm > 0.0 && d.height_mm > 0.0 {
        format!("{:.0} × {:.0} mm", d.width_mm, d.height_mm)
    } else {
        UNKNOWN.into()
    };
    rows.push(("Physical size", size));
    rows.push((
        "Diagonal",
        d.diagonal_inches().map_or(UNKNOWN.into(), |i| {
            format!("{i:.1} in ({:.0} cm)", i * 2.54)
        }),
    ));
    let (dpi_x, dpi_y) = match d.dpi() {
        Some((x, y)) => (format!("{x:.1}"), format!("{y:.1}")),
        None => (UNKNOWN.into(), UNKNOWN.into()),
    };
    rows.push(("Pixels per inch (X)", dpi_x));
    rows.push(("Pixels per inch (Y)", dpi_y));
    rows.push((
        "Pixel aspect ratio",
        d.pixel_aspect()
            .map_or(UNKNOWN.into(), |a| format!("{a:.3}")),
    ));
    rows.push((
        "Screen aspect ratio",
        d.screen_aspect().map_or(UNKNOWN.into(), |(a, b)| {
            let ratio = f64::from(a) / f64::from(b);
            format!("{a}:{b} ({ratio:.2})")
        }),
    ));
    rows.push((
        "Diagonal in pixels",
        format!("{:.0} px", d.diagonal_pixels()),
    ));
    rows.push((
        "Position",
        format!(
            "({}, {}) – ({}, {})",
            d.x,
            d.y,
            i64::from(d.x) + i64::from(d.width),
            i64::from(d.y) + i64::from(d.height)
        ),
    ));
    rows.push(("Rotation", format!("{:.0}°", d.rotation)));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_for_a_known_display() {
        let d = DisplayInfo {
            name: "Studio".into(),
            primary: true,
            width: 5120,
            height: 2880,
            scale: 2.0,
            refresh_mhz: 59_940,
            bits: 32,
            width_mm: 596.7,
            height_mm: 335.7,
            ..Default::default()
        };
        let rows = properties(&d);
        let get = |k| rows.iter().find(|(p, _)| *p == k).map(|(_, v)| v.as_str());
        assert_eq!(get("Resolution"), Some("5120 × 2880 px"));
        assert_eq!(get("Desktop resolution"), Some("2560 × 1440 pt"));
        assert_eq!(get("Refresh rate"), Some("59.94 Hz"));
        assert_eq!(get("Colours"), Some("16\u{202F}777\u{202F}216"));
        assert_eq!(get("Diagonal"), Some("27.0 in (68 cm)"));
        assert_eq!(get("Pixels per inch (X)"), Some("217.9"));
        assert_eq!(get("Screen aspect ratio"), Some("16:9 (1.78)"));
        assert_eq!(get("Position"), Some("(0, 0) – (5120, 2880)"));
        assert_eq!(list_label(&d), "Studio  —  5120 × 2880  (primary)");
    }

    #[test]
    fn unknowns_show_a_dash() {
        let d = DisplayInfo {
            width: 1920,
            height: 1080,
            scale: 1.0,
            ..Default::default()
        };
        let rows = properties(&d);
        for key in [
            "Name",
            "Refresh rate",
            "Colours",
            "Physical size",
            "Diagonal",
        ] {
            let v = rows
                .iter()
                .find(|(p, _)| *p == key)
                .map(|(_, v)| v.as_str());
            assert_eq!(v, Some(UNKNOWN), "{key}");
        }
        assert_eq!(refresh_text(None), UNKNOWN);
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1\u{202F}000");
    }
}
