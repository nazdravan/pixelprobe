use super::*;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

/// Draw `show` headlessly with an engine in `screen`; return the clicked actions.
fn click(screen_cmd: Option<&str>, label: &str) -> Vec<Action> {
    let mut engine = Engine::new();
    if let Some(id) = screen_cmd {
        engine.run(id, Value::Null).unwrap();
    }
    let mut harness = Harness::new_ui_state(
        |ui, actions: &mut Vec<Action>| actions.extend(show(ui, &engine)),
        Vec::new(),
    );
    // A fixed number of frames: animated screens repaint forever and never settle.
    harness.run_steps(2);
    harness.get_by_label(label).click();
    harness.run_steps(4);
    harness.state().clone()
}

#[test]
fn main_window_buttons_emit_commands() {
    assert_eq!(click(None, "Motion Test"), vec![Action::new("test.motion")]);
    assert_eq!(
        click(None, "Pattern Test"),
        vec![Action::new("test.pattern")]
    );
    assert_eq!(
        click(None, "Pixel Refresh"),
        vec![Action::new("test.pixel_refresh")]
    );
    assert_eq!(click(None, "About..."), vec![Action::new("app.about")]);
}

#[test]
fn every_screen_renders_and_closes() {
    for id in ["test.motion", "test.pixel_refresh", "app.about", "app.help"] {
        assert_eq!(
            click(Some(id), "Close"),
            vec![Action::new("view.close")],
            "{id}"
        );
    }
}

/// Press `key` (with `modifiers`) on a fresh frame; return the triggered actions.
fn press(screen_cmd: Option<&str>, modifiers: egui::Modifiers, key: egui::Key) -> Vec<Action> {
    let mut engine = Engine::new();
    if let Some(id) = screen_cmd {
        engine.run(id, Value::Null).unwrap();
    }
    let mut harness = Harness::new_ui_state(
        |ui, actions: &mut Vec<Action>| actions.extend(show(ui, &engine)),
        Vec::new(),
    );
    harness.key_press_modifiers(modifiers, key);
    harness.run_steps(4);
    harness.state().clone()
}

#[test]
fn shortcuts_trigger_enabled_commands() {
    let none = egui::Modifiers::NONE;
    assert_eq!(
        press(None, none, egui::Key::M),
        vec![Action::new("test.motion")]
    );
    assert_eq!(
        press(None, none, egui::Key::P),
        vec![Action::new("test.pattern")]
    );
    assert_eq!(
        press(Some("test.motion"), none, egui::Key::Escape),
        vec![Action::new("view.close")]
    );
    // Disabled: Escape on the main window, undo with empty history.
    assert_eq!(press(None, none, egui::Key::Escape), vec![]);
    assert_eq!(press(None, egui::Modifiers::COMMAND, egui::Key::Z), vec![]);
}

#[test]
fn pattern_screen_toolbar_and_keys() {
    let none = egui::Modifiers::NONE;
    assert_eq!(
        press(Some("test.pattern"), none, egui::Key::ArrowRight),
        vec![Action::new("pattern.next")]
    );
    assert_eq!(
        press(Some("test.pattern"), none, egui::Key::ArrowLeft),
        vec![Action::new("pattern.prev")]
    );
    assert_eq!(
        press(Some("test.pattern"), none, egui::Key::Escape),
        vec![Action::new("view.close")]
    );
    // Arrows do nothing outside the pattern test.
    assert_eq!(press(None, none, egui::Key::ArrowRight), vec![]);
}

#[test]
fn pattern_toolbar_buttons() {
    for (label, id) in [
        ("Next pattern (Right)", "pattern.next"),
        ("Previous pattern (Left)", "pattern.prev"),
    ] {
        let mut engine = Engine::new();
        engine.run("test.pattern", Value::Null).unwrap();
        let mut harness = Harness::new_ui_state(
            |ui, actions: &mut Vec<Action>| actions.extend(show(ui, &engine)),
            Vec::new(),
        );
        harness.run();
        // Icon buttons are found by their glyph; the label is the tooltip.
        let glyph = if id == "pattern.next" { "›" } else { "‹" };
        harness.get_by_label(glyph).click();
        harness.run();
        assert_eq!(harness.state(), &vec![Action::new(id)], "{label}");
    }
}

#[test]
fn pixel_refresh_screen() {
    let none = egui::Modifiers::NONE;
    assert_eq!(
        press(Some("test.pixel_refresh"), none, egui::Key::ArrowUp),
        vec![Action::new("refresh.faster")]
    );
    assert_eq!(
        press(Some("test.pixel_refresh"), none, egui::Key::ArrowDown),
        vec![Action::new("refresh.slower")]
    );
    assert_eq!(press(None, none, egui::Key::ArrowUp), vec![]);

    let mut engine = Engine::new();
    engine.run("test.pixel_refresh", Value::Null).unwrap();
    engine
        .run("refresh.speed", pp_engine::json!({"speed": 3}))
        .unwrap();
    let mut harness = Harness::new_ui_state(
        |ui, actions: &mut Vec<Action>| actions.extend(show(ui, &engine)),
        Vec::new(),
    );
    harness.run();
    harness.get_by_label("Update rate: 5 /s");
    harness.get_by_label("Elapsed time: 00:00");
    // Run frames as the animating app does, then check the layout.
    for _ in 0..60 {
        harness.step();
    }
    // Layout: noise on top, then the controls, then Close at the bottom.
    let speed = harness.get_by_label("Speed:").rect();
    let close = harness.get_by_label("Close").rect();
    assert!(speed.min.y > 100.0, "noise above the controls: {speed:?}");
    assert!(
        close.min.y > speed.max.y,
        "Close below the controls: {close:?}"
    );
    assert!(harness.state().is_empty());
}

#[test]
fn refresh_clock_format() {
    use crate::pixel_refresh::clock;
    assert_eq!(clock(0.0), "00:00");
    assert_eq!(clock(4.9), "00:04");
    assert_eq!(clock(61.0), "01:01");
    assert_eq!(clock(3599.0), "59:59");
    assert_eq!(clock(3600.0), "1:00:00");
    assert_eq!(clock(-5.0), "00:00");
    assert_eq!(clock(f64::NAN), "00:00");
    assert_eq!(clock(f64::INFINITY), "00:00");
}

#[test]
fn motion_screen_controls() {
    let m = Some("test.motion");
    assert_eq!(
        click(m, "Circle"),
        vec![Action {
            id: "motion.set",
            params: pp_engine::json!({"shape": "circle"})
        }]
    );
    assert_eq!(
        click(m, "Large"),
        vec![Action {
            id: "motion.set",
            params: pp_engine::json!({"size": "large"})
        }]
    );
    assert_eq!(
        click(m, "Shape by shape"),
        vec![Action {
            id: "motion.set",
            params: pp_engine::json!({"step": "shape_by_shape"})
        }]
    );
    assert_eq!(
        click(m, "Hollow"),
        vec![Action {
            id: "motion.set",
            params: pp_engine::json!({"hollow": true})
        }]
    );
    // Already selected: no command.
    assert_eq!(click(m, "Square"), vec![]);
    assert_eq!(click(m, "+"), vec![Action::new("motion.faster")]);
    assert_eq!(click(m, "-"), vec![Action::new("motion.slower")]);
    assert_eq!(
        click(m, "Swap colours"),
        vec![Action::new("motion.swap_colors")]
    );
    let none = egui::Modifiers::NONE;
    assert_eq!(
        press(m, none, egui::Key::ArrowUp),
        vec![Action::new("motion.faster")]
    );
    assert_eq!(
        press(m, none, egui::Key::S),
        vec![Action::new("motion.swap_colors")]
    );
}

#[test]
fn monitor_list_and_properties() {
    use pp_engine::pp_core::DisplayInfo;
    let mut engine = Engine::new();
    engine.set_displays(vec![
        DisplayInfo {
            name: "Left".into(),
            primary: true,
            width: 2560,
            height: 1440,
            scale: 1.0,
            // Every property row filled, so the font check covers all their glyphs.
            width_mm: 597.0,
            height_mm: 336.0,
            bits: 32,
            refresh_mhz: 60_000,
            modes: vec![
                pp_engine::pp_core::DisplayMode {
                    width: 1280,
                    height: 720,
                    bits: 32,
                    refresh_mhz: 60_000,
                },
                pp_engine::pp_core::DisplayMode {
                    width: 2560,
                    height: 1440,
                    bits: 32,
                    refresh_mhz: 60_000,
                },
            ],
            ..Default::default()
        },
        DisplayInfo {
            name: "Right".into(),
            width: 1920,
            height: 1080,
            scale: 1.0,
            refresh_mhz: 144_000,
            ..Default::default()
        },
    ]);
    let mut harness = Harness::new_ui_state(
        |ui, actions: &mut Vec<Action>| actions.extend(show(ui, &engine)),
        Vec::new(),
    );
    harness.run();
    // Properties of the selected (first) display.
    harness.get_by_label("2560 × 1440 px");
    harness.get_by_label("current");
    harness.get_by_label("1280 × 720").click();
    harness.get_by_label("Right  —  1920 × 1080").click();
    harness.get_by_label("Refresh").click();
    harness.run();
    assert_eq!(
        harness.state(),
        &vec![
            Action {
                id: "display.select_mode",
                params: pp_engine::json!({"index": 0})
            },
            Action {
                id: "display.select",
                params: pp_engine::json!({"index": 1})
            },
            Action::new("display.refresh"),
        ]
    );
}

#[test]
fn help_button_key_and_shortcut_table() {
    assert_eq!(click(None, "?"), vec![Action::new("app.help")]);
    assert_eq!(
        press(None, egui::Modifiers::NONE, egui::Key::F1),
        vec![Action::new("app.help")]
    );
    let engine = Engine::new();
    let rows = crate::help::shortcut_rows(&engine);
    assert!(rows.contains(&("F1", "Help/Help")));
    assert!(rows.contains(&("Escape", "View/Close")));
    // Every shortcut in the table parses, so the table never lists a dead key.
    for (key, _) in &rows {
        assert!(crate::shortcuts::parse(key).is_some(), "{key}");
    }
}

#[test]
fn test_mode_checkbox() {
    assert_eq!(
        click(None, "Set test mode as selection"),
        vec![Action {
            id: "display.test_in_mode",
            params: pp_engine::json!({"on": true})
        }]
    );
}

#[test]
fn pattern_test_hides_toolbar_and_cursor_when_idle() {
    let mut engine = Engine::new();
    engine.run("test.pattern", Value::Null).unwrap();
    let mut harness = Harness::new_ui_state(
        |ui, actions: &mut Vec<Action>| actions.extend(show(ui, &engine)),
        Vec::new(),
    );
    harness.run_steps(2);
    let cursor = |h: &Harness<'_, Vec<Action>>| h.output().platform_output.cursor_icon;
    assert_ne!(cursor(&harness), egui::CursorIcon::None, "just opened");
    harness.get_by_label("›");
    // Five seconds later (kittest advances 0.25 s per frame) with no pointer movement.
    for _ in 0..20 {
        harness.step();
    }
    assert_eq!(cursor(&harness), egui::CursorIcon::None);
    assert!(harness.query_by_label("›").is_none(), "toolbar hidden");
}

#[test]
fn smpte_variant_button_and_key() {
    let p = Some("test.pattern");
    assert_eq!(
        click(p, "*2: 75% white"),
        vec![Action::new("pattern.smpte_next")]
    );
    assert_eq!(
        press(p, egui::Modifiers::NONE, egui::Key::V),
        vec![Action::new("pattern.smpte_next")]
    );
}

#[test]
fn pixel_refresh_full_screen() {
    let r = Some("test.pixel_refresh");
    assert_eq!(
        click(r, "Full screen"),
        vec![Action::new("refresh.full_screen")]
    );
    assert_eq!(
        press(r, egui::Modifiers::NONE, egui::Key::F),
        vec![Action::new("refresh.full_screen")]
    );
    let mut engine = Engine::new();
    engine.run("test.pixel_refresh", Value::Null).unwrap();
    engine.run("refresh.full_screen", Value::Null).unwrap();
    let mut harness = Harness::new_ui_state(
        |ui, actions: &mut Vec<Action>| actions.extend(show(ui, &engine)),
        Vec::new(),
    );
    harness.run_steps(2);
    harness.get_by_label("F: leave full screen    Esc: close");
    assert!(
        harness.query_by_label("Speed:").is_none(),
        "controls hidden"
    );
    for _ in 0..20 {
        harness.step();
    }
    assert!(
        harness
            .query_by_label("F: leave full screen    Esc: close")
            .is_none()
    );
}
