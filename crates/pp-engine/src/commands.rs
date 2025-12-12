//! The app's commands, grouped by area.

use crate::registry::always_enabled;
use crate::{CommandSpec, Registry, Session, Value};
use pp_core::Screen;

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
    use pp_core::Screen;

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
    fn app_commands_have_menus_and_labels() {
        let e = Engine::new();
        for c in e.registry().iter() {
            assert!(!c.label.is_empty() && !c.menu.is_empty(), "{}", c.id);
        }
    }
}
