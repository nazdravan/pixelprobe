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
    for id in [
        "test.motion",
        "test.pattern",
        "test.pixel_refresh",
        "app.about",
    ] {
        assert_eq!(
            click(Some(id), "Close"),
            vec![Action::new("view.close")],
            "{id}"
        );
    }
}
