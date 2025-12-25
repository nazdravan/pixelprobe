//! The app's commands, grouped by area.

use crate::registry::always_enabled;
use crate::{CommandSpec, Registry, Session, Value};
use pp_core::{MotionSpeed, MoveStep, PatternId, RefreshSpeed, Rgb8, Screen, Shape, ShapeSize};

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
    register_refresh(r);
    register_motion(r);
    register_display(r);
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

fn on_refresh_screen(s: &Session) -> Result<(), String> {
    if s.doc.screen == Screen::PixelRefresh {
        Ok(())
    } else {
        Err("the pixel refresh is not open".into())
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SetSpeed {
    speed: RefreshSpeed,
}

/// Move the refresh speed by `step`, failing at either end of the range.
fn step_speed(s: &mut Session, id: &str, step: i8) -> Result<Value, crate::EngineError> {
    let next = s.doc.refresh_speed.get().saturating_add_signed(step);
    s.doc.refresh_speed = RefreshSpeed::new(next).map_err(|msg| crate::EngineError::Failed {
        id: id.to_owned(),
        msg,
    })?;
    Ok(Value::Null)
}

fn register_refresh(r: &mut Registry) {
    r.register(CommandSpec {
        id: "refresh.speed",
        label: "Set refresh speed",
        menu: "Pixel Refresh/Speed...",
        shortcut: None,
        params: "{ speed: 1..=10 }",
        undoable: true,
        enabled: always_enabled,
        run: |s, p| {
            let p: SetSpeed = crate::params("refresh.speed", p)?;
            s.doc.refresh_speed = p.speed;
            Ok(Value::Null)
        },
    });
    r.register(CommandSpec {
        id: "refresh.faster",
        label: "Faster",
        menu: "Pixel Refresh/Faster",
        shortcut: Some("Up"),
        params: "",
        undoable: true,
        enabled: |s| {
            on_refresh_screen(s)?;
            if s.doc.refresh_speed.get() >= RefreshSpeed::MAX {
                return Err("already at the fastest speed".into());
            }
            Ok(())
        },
        run: |s, _| step_speed(s, "refresh.faster", 1),
    });
    r.register(CommandSpec {
        id: "refresh.slower",
        label: "Slower",
        menu: "Pixel Refresh/Slower",
        shortcut: Some("Down"),
        params: "",
        undoable: true,
        enabled: |s| {
            on_refresh_screen(s)?;
            if s.doc.refresh_speed.get() <= RefreshSpeed::MIN {
                return Err("already at the slowest speed".into());
            }
            Ok(())
        },
        run: |s, _| step_speed(s, "refresh.slower", -1),
    });
}

fn on_motion_screen(s: &Session) -> Result<(), String> {
    if s.doc.screen == Screen::MotionTest {
        Ok(())
    } else {
        Err("the motion test is not open".into())
    }
}

/// Any subset of the motion options; absent fields stay as they are.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MotionPatch {
    speed: Option<MotionSpeed>,
    shape: Option<Shape>,
    hollow: Option<bool>,
    size: Option<ShapeSize>,
    shape_color: Option<Rgb8>,
    background: Option<Rgb8>,
    step: Option<MoveStep>,
}

fn step_motion(s: &mut Session, id: &str, step: i8) -> Result<Value, crate::EngineError> {
    let next = s.doc.motion.speed.get().saturating_add_signed(step);
    s.doc.motion.speed = MotionSpeed::new(next).map_err(|msg| crate::EngineError::Failed {
        id: id.to_owned(),
        msg,
    })?;
    Ok(Value::Null)
}

fn register_motion(r: &mut Registry) {
    r.register(CommandSpec {
        id: "motion.set",
        label: "Set motion options",
        menu: "Motion/Options...",
        shortcut: None,
        params: "{ speed?: 1..=12, shape?: \"square\" | \"circle\", hollow?: bool, size?: \"small\" | \"medium\" | \"large\", shape_color?: {r,g,b}, background?: {r,g,b}, step?: \"pixel_by_pixel\" | \"shape_by_shape\" }",
        undoable: true,
        enabled: always_enabled,
        run: |s, p| {
            let p: MotionPatch = crate::params("motion.set", p)?;
            let m = &mut s.doc.motion;
            if let Some(v) = p.speed {
                m.speed = v;
            }
            if let Some(v) = p.shape {
                m.shape = v;
            }
            if let Some(v) = p.hollow {
                m.hollow = v;
            }
            if let Some(v) = p.size {
                m.size = v;
            }
            if let Some(v) = p.shape_color {
                m.shape_color = v;
            }
            if let Some(v) = p.background {
                m.background = v;
            }
            if let Some(v) = p.step {
                m.step = v;
            }
            Ok(Value::Null)
        },
    });
    r.register(CommandSpec {
        id: "motion.swap_colors",
        label: "Swap colours",
        menu: "Motion/Swap Colours",
        shortcut: Some("S"),
        params: "",
        undoable: true,
        enabled: on_motion_screen,
        run: |s, _| {
            let m = &mut s.doc.motion;
            std::mem::swap(&mut m.shape_color, &mut m.background);
            Ok(Value::Null)
        },
    });
    r.register(CommandSpec {
        id: "motion.faster",
        label: "+",
        menu: "Motion/Faster",
        shortcut: Some("Up"),
        params: "",
        undoable: true,
        enabled: |s| {
            on_motion_screen(s)?;
            if s.doc.motion.speed.get() >= MotionSpeed::MAX {
                return Err("already at the fastest speed".into());
            }
            Ok(())
        },
        run: |s, _| step_motion(s, "motion.faster", 1),
    });
    r.register(CommandSpec {
        id: "motion.slower",
        label: "-",
        menu: "Motion/Slower",
        shortcut: Some("Down"),
        params: "",
        undoable: true,
        enabled: |s| {
            on_motion_screen(s)?;
            if s.doc.motion.speed.get() <= MotionSpeed::MIN {
                return Err("already at the slowest speed".into());
            }
            Ok(())
        },
        run: |s, _| step_motion(s, "motion.slower", -1),
    });
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectDisplay {
    index: usize,
}

fn register_display(r: &mut Registry) {
    r.register(CommandSpec {
        id: "display.refresh",
        label: "Refresh",
        menu: "Display/Refresh",
        shortcut: Some("F5"),
        params: "",
        undoable: false,
        enabled: |s| {
            if s.doc.screen == Screen::Main {
                Ok(())
            } else {
                Err("the monitor list is on the main window".into())
            }
        },
        run: |s, _| {
            s.display_refreshes = s.display_refreshes.wrapping_add(1);
            Ok(Value::Null)
        },
    });
    r.register(CommandSpec {
        id: "display.select",
        label: "Select monitor",
        menu: "Display/Select...",
        shortcut: None,
        params: "{ index: number } (0-based, in the monitor list)",
        undoable: false,
        enabled: |s| {
            if s.displays.is_empty() {
                Err("no displays detected".into())
            } else {
                Ok(())
            }
        },
        run: |s, p| {
            let p: SelectDisplay = crate::params("display.select", p)?;
            if p.index >= s.displays.len() {
                return Err(crate::EngineError::Failed {
                    id: "display.select".into(),
                    msg: format!("no display {} (there are {})", p.index, s.displays.len()),
                });
            }
            if s.doc.monitor != p.index {
                s.doc.monitor = p.index;
                s.doc.mode = None;
            }
            Ok(Value::Null)
        },
    });
    r.register(CommandSpec {
        id: "display.select_mode",
        label: "Select display mode",
        menu: "Display/Select Mode...",
        shortcut: None,
        params: "{ index: number } (0-based, in the selected monitor's mode list)",
        undoable: false,
        enabled: |s| match s.displays.get(s.doc.monitor) {
            Some(d) if !d.modes.is_empty() => Ok(()),
            Some(_) => Err("the selected monitor reports no modes".into()),
            None => Err("no displays detected".into()),
        },
        run: |s, p| {
            let p: SelectDisplay = crate::params("display.select_mode", p)?;
            let count = s.displays.get(s.doc.monitor).map_or(0, |d| d.modes.len());
            if p.index >= count {
                return Err(crate::EngineError::Failed {
                    id: "display.select_mode".into(),
                    msg: format!("no mode {} (there are {count})", p.index),
                });
            }
            s.doc.mode = Some(p.index);
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
    fn refresh_speed_commands_and_undo() {
        let mut e = Engine::new();
        assert!(e.enabled("refresh.faster").is_err(), "only on its screen");
        e.run("test.pixel_refresh", Value::Null).unwrap();
        assert_eq!(e.doc().refresh_speed.get(), 7);
        e.run("refresh.faster", Value::Null).unwrap();
        assert_eq!(e.doc().refresh_speed.get(), 8);
        e.run("refresh.speed", serde_json::json!({"speed": 10}))
            .unwrap();
        assert!(e.enabled("refresh.faster").is_err(), "at the top");
        for bad in [
            serde_json::json!({"speed": 0}),
            serde_json::json!({"speed": 11}),
            serde_json::json!({"speed": "fast"}),
            serde_json::json!({"speed": 5, "x": 1}),
            Value::Null,
        ] {
            assert!(e.run("refresh.speed", bad.clone()).is_err(), "{bad}");
        }
        assert_eq!(e.doc().refresh_speed.get(), 10);
        e.run("refresh.speed", serde_json::json!({"speed": 1}))
            .unwrap();
        assert!(e.enabled("refresh.slower").is_err(), "at the bottom");

        // Undo walks back 1 → 10 → 8 → 7 and never leaves the current screen.
        e.run("view.close", Value::Null).unwrap();
        e.run("edit.undo", Value::Null).unwrap();
        assert_eq!(e.doc().refresh_speed.get(), 10);
        assert_eq!(e.doc().screen, Screen::Main);
        e.run("edit.undo", Value::Null).unwrap();
        e.run("edit.undo", Value::Null).unwrap();
        assert_eq!(e.doc().refresh_speed.get(), 7);
        assert!(e.enabled("edit.undo").is_err());
        e.run("edit.redo", Value::Null).unwrap();
        assert_eq!(e.doc().refresh_speed.get(), 8);
        assert_eq!(e.doc().screen, Screen::Main);
    }

    #[test]
    fn motion_commands() {
        use pp_core::{MoveStep, Rgb8, Shape, ShapeSize};
        let mut e = Engine::new();
        assert!(e.enabled("motion.faster").is_err(), "only on its screen");
        e.run("test.motion", Value::Null).unwrap();
        e.run(
            "motion.set",
            serde_json::json!({"shape": "circle", "hollow": true, "size": "large",
                "step": "shape_by_shape", "shape_color": {"r": 255, "g": 0, "b": 0}}),
        )
        .unwrap();
        let m = e.doc().motion;
        assert_eq!(
            (m.shape, m.hollow, m.size, m.step),
            (
                Shape::Circle,
                true,
                ShapeSize::Large,
                MoveStep::ShapeByShape
            )
        );
        assert_eq!(m.shape_color, Rgb8::new(255, 0, 0));
        e.run("motion.swap_colors", Value::Null).unwrap();
        assert_eq!(e.doc().motion.background, Rgb8::new(255, 0, 0));
        assert_eq!(e.doc().motion.shape_color, Rgb8::BLACK);
        e.run("motion.faster", Value::Null).unwrap();
        assert_eq!(e.doc().motion.speed.rate_hz(), 48);
        for bad in [
            serde_json::json!({"speed": 0}),
            serde_json::json!({"speed": 13}),
            serde_json::json!({"shape": "triangle"}),
            serde_json::json!({"shape_color": {"r": 256, "g": 0, "b": 0}}),
            serde_json::json!({"bogus": 1}),
            Value::Null,
        ] {
            assert!(e.run("motion.set", bad.clone()).is_err(), "{bad}");
        }
        e.run("motion.set", serde_json::json!({"speed": 12}))
            .unwrap();
        assert!(e.enabled("motion.faster").is_err());
        // Undo everything: back to the defaults, still on the motion screen.
        while e.enabled("edit.undo").is_ok() {
            e.run("edit.undo", Value::Null).unwrap();
        }
        assert_eq!(e.doc().motion, pp_core::Motion::default());
        assert_eq!(e.doc().screen, Screen::MotionTest);
    }

    #[test]
    fn display_list_and_selection() {
        use pp_core::DisplayInfo;
        let mut e = Engine::new();
        assert!(e.enabled("display.select").is_err(), "no displays yet");
        let named = |n: &str| DisplayInfo {
            name: n.into(),
            width: 1920,
            height: 1080,
            ..Default::default()
        };
        e.set_displays(vec![named("A"), named("B")]);
        e.run("display.select", serde_json::json!({"index": 1}))
            .unwrap();
        assert_eq!(e.selected_display().unwrap().name, "B");
        assert!(
            e.run("display.select", serde_json::json!({"index": 2}))
                .is_err()
        );
        assert!(
            e.run("display.select", serde_json::json!({"index": -1}))
                .is_err()
        );
        // A display unplugged: the selection falls back to the first.
        e.set_displays(vec![named("A")]);
        assert_eq!(e.doc().monitor, 0);
        // Bounded input.
        e.set_displays(vec![named("X"); 100]);
        assert_eq!(e.displays().len(), pp_core::display::MAX_DISPLAYS);

        // Modes: select one, then switching monitor clears it.
        let with_modes = |n: &str| DisplayInfo {
            name: n.into(),
            width: 800,
            height: 600,
            modes: vec![
                pp_core::DisplayMode {
                    width: 800,
                    height: 600,
                    bits: 32,
                    refresh_mhz: 60_000
                };
                1
            ],
            ..Default::default()
        };
        e.set_displays(vec![with_modes("A"), named("B")]);
        e.run("display.select_mode", serde_json::json!({"index": 0}))
            .unwrap();
        assert_eq!(e.doc().mode, Some(0));
        assert!(
            e.run("display.select_mode", serde_json::json!({"index": 1}))
                .is_err()
        );
        e.run("display.select", serde_json::json!({"index": 1}))
            .unwrap();
        assert_eq!(e.doc().mode, None);
        assert!(e.enabled("display.select_mode").is_err(), "B has no modes");
        e.set_displays(vec![with_modes("A")]);

        let before = e.session().display_refreshes;
        e.run("display.refresh", Value::Null).unwrap();
        assert_eq!(e.session().display_refreshes, before + 1);
        e.run("test.motion", Value::Null).unwrap();
        assert!(e.enabled("display.refresh").is_err());
    }

    #[test]
    fn app_commands_have_menus_and_labels() {
        let e = Engine::new();
        for c in e.registry().iter() {
            assert!(!c.label.is_empty() && !c.menu.is_empty(), "{}", c.id);
        }
    }
}
