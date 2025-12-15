//! Keyboard shortcuts: the strings in `CommandSpec::shortcut` mapped to egui key events.
//! This is the only place that knows the shortcut syntax.

use crate::Action;
use pp_engine::Engine;

/// Parse `"Cmd+Shift+Z"`, `"Escape"`, `"Right"`, `"M"`… `Cmd` is ⌘ on macOS, Ctrl elsewhere.
/// Returns `None` for anything malformed.
pub fn parse(spec: &str) -> Option<egui::KeyboardShortcut> {
    if spec.is_empty() || spec.len() > 64 {
        return None;
    }
    let mut parts: Vec<&str> = spec.split('+').collect();
    let key_name = parts.pop()?;
    let key = egui::Key::from_name(key_name)?;
    let mut modifiers = egui::Modifiers::NONE;
    for part in parts {
        let m = match part {
            "Cmd" | "Ctrl" => egui::Modifiers::COMMAND,
            "Shift" => egui::Modifiers::SHIFT,
            "Alt" => egui::Modifiers::ALT,
            _ => return None,
        };
        if modifiers.contains(m) {
            return None; // repeated modifier
        }
        modifiers |= m;
    }
    Some(egui::KeyboardShortcut::new(modifiers, key))
}

/// Consume the shortcuts pressed this frame and return the commands they trigger.
/// Disabled commands are skipped; nothing fires while a text field has focus.
pub fn actions(ctx: &egui::Context, engine: &Engine) -> Vec<Action> {
    if ctx.egui_wants_keyboard_input() {
        return Vec::new();
    }
    let mut bound: Vec<(egui::KeyboardShortcut, &'static str)> = engine
        .registry()
        .iter()
        .filter_map(|c| Some((parse(c.shortcut?)?, c.id)))
        .collect();
    // Check the most specific first so Cmd+Shift+Z is not eaten by Cmd+Z.
    bound.sort_by_key(|(s, _)| std::cmp::Reverse(modifier_count(s.modifiers)));
    let mut out = Vec::new();
    for (shortcut, id) in bound {
        if engine.enabled(id).is_ok() && ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
            out.push(Action::new(id));
        }
    }
    out
}

fn modifier_count(m: egui::Modifiers) -> u8 {
    u8::from(m.command || m.ctrl || m.mac_cmd) + u8::from(m.shift) + u8::from(m.alt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_known_forms() {
        let s = parse("Cmd+Shift+Z").unwrap();
        assert_eq!(s.logical_key, egui::Key::Z);
        assert!(s.modifiers.command && s.modifiers.shift && !s.modifiers.alt);
        assert_eq!(parse("Escape").unwrap().logical_key, egui::Key::Escape);
        assert_eq!(parse("Right").unwrap().logical_key, egui::Key::ArrowRight);
        assert_eq!(parse("M").unwrap().modifiers, egui::Modifiers::NONE);
    }

    #[test]
    fn rejects_malformed() {
        for bad in [
            "",
            "+",
            "Cmd+",
            "Hyper+Z",
            "Cmd+Cmd+Z",
            "NoSuchKey",
            "Z+Cmd",
        ] {
            assert!(parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn every_registered_shortcut_parses() {
        let engine = Engine::new();
        for c in engine.registry().iter() {
            if let Some(s) = c.shortcut {
                assert!(parse(s).is_some(), "{}: {s}", c.id);
            }
        }
    }

    proptest! {
        #[test]
        fn parse_never_panics(s in ".{0,80}") {
            let _ = parse(&s);
        }
    }
}
