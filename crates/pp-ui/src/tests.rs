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
    harness.get_by_label(label).click();
    harness.run();
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
    for id in ["test.motion", "test.pixel_refresh", "app.about"] {
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
    harness.run();
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
