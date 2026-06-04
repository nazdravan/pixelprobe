//! The app's colours, taken from `assets/theme.toml` (a test keeps the two equal).

use egui::{Color32, Context, Stroke, Theme, Visuals};

pub const ACCENT: Color32 = Color32::from_rgb(0x0e, 0x74, 0x90);
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(0x22, 0xd3, 0xee);
pub const BG_DARK: Color32 = Color32::from_rgb(0x0d, 0x10, 0x17);
pub const PANEL_DARK: Color32 = Color32::from_rgb(0x15, 0x1a, 0x23);
pub const FG_DARK: Color32 = Color32::from_rgb(0xe6, 0xe9, 0xef);
pub const BG_LIGHT: Color32 = Color32::from_rgb(0xf6, 0xf7, 0xf9);
pub const PANEL_LIGHT: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
pub const FG_LIGHT: Color32 = Color32::from_rgb(0x14, 0x18, 0x20);
pub const BEZEL: Color32 = Color32::from_rgb(0x3a, 0x44, 0x57);

/// egui visuals for one theme, built on egui's own defaults.
pub fn visuals(theme: Theme) -> Visuals {
    let (mut v, bg, panel, fg, link) = match theme {
        Theme::Dark => (Visuals::dark(), BG_DARK, PANEL_DARK, FG_DARK, ACCENT_LIGHT),
        Theme::Light => (Visuals::light(), BG_LIGHT, PANEL_LIGHT, FG_LIGHT, ACCENT),
    };
    v.panel_fill = bg;
    v.window_fill = panel;
    v.faint_bg_color = panel;
    v.extreme_bg_color = match theme {
        Theme::Dark => Color32::BLACK,
        Theme::Light => PANEL_LIGHT,
    };
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.hyperlink_color = link;
    v.override_text_color = Some(fg);
    if theme == Theme::Dark {
        // Buttons in the monitor-bezel grey-blue instead of egui's neutral grey.
        v.widgets.inactive.weak_bg_fill = BEZEL;
        v.widgets.inactive.bg_fill = BEZEL;
    }
    v
}

/// Install both themes; egui keeps following the system's light/dark setting.
pub fn apply(ctx: &Context) {
    ctx.set_visuals_of(Theme::Dark, visuals(Theme::Dark));
    ctx.set_visuals_of(Theme::Light, visuals(Theme::Light));
}

#[cfg(test)]
mod tests {
    use super::*;

    const THEME_TOML: &str = include_str!("../../../assets/theme.toml");

    /// `key = "#rrggbb"` from the theme file.
    fn colour(key: &str) -> String {
        THEME_TOML
            .lines()
            .find_map(|l| {
                let (k, v) = l.split_once('=')?;
                (k.trim() == key).then(|| v.trim().trim_matches('"').to_owned())
            })
            .unwrap_or_else(|| panic!("{key} missing from theme.toml"))
    }

    fn hex(c: Color32) -> String {
        format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
    }

    #[test]
    fn constants_match_theme_toml() {
        for (key, c) in [
            ("accent", ACCENT),
            ("accent_light", ACCENT_LIGHT),
            ("bg_dark", BG_DARK),
            ("panel_dark", PANEL_DARK),
            ("fg_dark", FG_DARK),
            ("bg_light", BG_LIGHT),
            ("panel_light", PANEL_LIGHT),
            ("fg_light", FG_LIGHT),
            ("bezel", BEZEL),
        ] {
            assert_eq!(hex(c), colour(key), "{key}");
        }
    }

    #[test]
    fn web_page_background_comes_from_the_theme() {
        let html = include_str!("../../../app/web/index.html");
        for key in ["bg_dark", "bg_light"] {
            assert!(html.contains(&colour(key)), "{key} not in index.html");
        }
    }

    #[test]
    fn both_themes_use_the_accent() {
        for t in [Theme::Dark, Theme::Light] {
            let v = visuals(t);
            assert_eq!(v.selection.bg_fill, ACCENT);
            assert_eq!(v.dark_mode, t == Theme::Dark);
        }
        assert_eq!(visuals(Theme::Dark).panel_fill, BG_DARK);
        assert_eq!(visuals(Theme::Light).panel_fill, BG_LIGHT);
    }
}
