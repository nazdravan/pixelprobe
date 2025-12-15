//! The app's commands, grouped by area.

use crate::registry::always_enabled;
use crate::{CommandSpec, Registry, Session, Value};
use pp_core::{PatternId, Screen};

/// Register every app command on top of the built-ins.
pub fn register_app(r: &mut Registry) {
    for (id, label, menu, shortcut, screen) in [
        (
            "test.motion",
            "Motion Test",
            "Tests/Motion Test",
            Some("M"),
            Screen::MotionTest,
        ),
        (
            "test.pattern",
            "Pattern Test",
            "Tests/Pattern Test",
            Some("P"),
            Screen::PatternTest,
        ),
        (
            "test.pixel_refresh",
            "Pixel Refresh",
            "Tests/Pixel Refresh",
            Some("R"),
            Screen::PixelRefresh,
        ),
        ("app.about", "About...", "Help/About", None, Screen::About),
    ] {
        r.register(open_screen(id, label, menu, shortcut, screen));
    }
    r.register(CommandSpec {
        id: "view.close",
        label: "Close",
        menu: "View/Close",
        shortcut: Some("Escape"),
        params: "",
        undoable: false,
        enabled: |s| {
            if s.doc.screen == Screen::Main {
                Err("already on the main window".into())
            } else {
                Ok(())
            }
        },
        run: |s, _| {
            s.doc.screen = Screen::Main;
            Ok(Value::Null)
        },
    });
    register_patterns(r);
}

fn on_pattern_screen(s: &Session) -> Result<(), String> {
    if s.doc.screen == Screen::PatternTest {
        Ok(())
    } else {
        Err("the pattern test is not open".into())
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SetPattern {
    pattern: PatternId,
}

fn register_patterns(r: &mut Registry) {
    r.register(CommandSpec {
        id: "pattern.next",
        label: "Next pattern",
        menu: "Pattern/Next",
        shortcut: Some("Right"),
        params: "",
        undoable: false,
        enabled: on_pattern_screen,
        run: |s, _| {
            s.doc.pattern = s.doc.pattern.next();
            Ok(Value::Null)
        },
    });
    r.register(CommandSpec {
        id: "pattern.prev",
        label: "Previous pattern",
        menu: "Pattern/Previous",
        shortcut: Some("Left"),
        params: "",
        undoable: false,
        enabled: on_pattern_screen,
        run: |s, _| {
            s.doc.pattern = s.doc.pattern.prev();
            Ok(Value::Null)
        },
    });
    r.register(CommandSpec {
        id: "pattern.set",
        label: "Show pattern",
        menu: "Pattern/Show...",
        shortcut: None,
        params: "{ pattern: \"smpte_bars\" | \"white\" | \"black\" | \"red\" | \"green\" | \"blue\" | \"gray50\" | \"gray_ramp\" | \"gray_steps\" | \"rgb_ramps\" | \"black_level\" | \"white_level\" | \"grid\" | \"checker_fine\" | \"checker_coarse\" }",
        undoable: false,
        enabled: always_enabled,
        run: |s, p| {
            let p: SetPattern = crate::params("pattern.set", p)?;
            s.doc.pattern = p.pattern;
            Ok(Value::Null)
        },
    });
}

/// A command that brings `screen` to the front. Navigation is not undoable.
fn open_screen(
    id: &'static str,
    label: &'static str,
    menu: &'static str,
    shortcut: Option<&'static str>,
    screen: Screen,
) -> CommandSpec {
    // `run` is a plain fn pointer, so map the id back to its screen inside it.
    fn run(s: &mut Session, _: &Value, screen: Screen) -> Result<Value, crate::EngineError> {
        s.doc.screen = screen;
        Ok(Value::Null)
    }
    CommandSpec {
        id,
        label,
        menu,
        shortcut,
        params: "",
        undoable: false,
        enabled: always_enabled,
        run: match screen {
            Screen::MotionTest => |s, p| run(s, p, Screen::MotionTest),
            Screen::PatternTest => |s, p| run(s, p, Screen::PatternTest),
            Screen::PixelRefresh => |s, p| run(s, p, Screen::PixelRefresh),
            Screen::About => |s, p| run(s, p, Screen::About),
            Screen::Main => |s, p| run(s, p, Screen::Main),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{Engine, Value};
    use pp_core::{PatternId, Screen};

    #[test]
    fn open_and_close_each_screen() {
        let mut e = Engine::new();
        assert!(e.enabled("view.close").is_err());
        for (id, screen) in [
            ("test.motion", Screen::MotionTest),
            ("test.pattern", Screen::PatternTest),
            ("test.pixel_refresh", Screen::PixelRefresh),
            ("app.about", Screen::About),
        ] {
            e.run(id, Value::Null).unwrap();
            assert_eq!(e.doc().screen, screen, "{id}");
            e.run("view.close", Value::Null).unwrap();
            assert_eq!(e.doc().screen, Screen::Main);
        }
        // Navigation is not undoable.
        assert!(!e.session().history.can_undo());
    }

    #[test]
    fn pattern_navigation() {
        let mut e = Engine::new();
        assert!(
            e.enabled("pattern.next").is_err(),
            "only on the pattern screen"
        );
        e.run("test.pattern", Value::Null).unwrap();
        assert_eq!(e.doc().pattern, PatternId::SmpteBars);
        e.run("pattern.next", Value::Null).unwrap();
        assert_eq!(e.doc().pattern, PatternId::White);
        e.run("pattern.prev", Value::Null).unwrap();
        e.run("pattern.prev", Value::Null).unwrap();
        assert_eq!(e.doc().pattern, PatternId::CheckerCoarse);
        e.run("pattern.set", serde_json::json!({"pattern": "grid"}))
            .unwrap();
        assert_eq!(e.doc().pattern, PatternId::Grid);
        assert!(
            e.run("pattern.set", serde_json::json!({"pattern": "nope"}))
                .is_err()
        );
        assert_eq!(e.doc().pattern, PatternId::Grid);
    }

    #[test]
    fn app_commands_have_menus_and_labels() {
        let e = Engine::new();
        for c in e.registry().iter() {
            assert!(!c.label.is_empty() && !c.menu.is_empty(), "{}", c.id);
        }
    }
}
