use crate::Action;
use pp_engine::Engine;

/// A button for a registered command: label from the spec, greyed out with the reason as a
/// tooltip when disabled, shortcut in the tooltip. A click pushes an [`Action`].
pub fn command_button(
    ui: &mut egui::Ui,
    engine: &Engine,
    id: &'static str,
    actions: &mut Vec<Action>,
) -> egui::Response {
    let Some(spec) = engine.registry().get(id) else {
        // A wrong id is a programming error; show it instead of crashing.
        return ui.label(format!("unknown command {id}"));
    };
    let enabled = engine.enabled(id);
    let button = egui::Button::new(spec.label).min_size(egui::vec2(96.0, 24.0));
    let mut resp = ui.add_enabled(enabled.is_ok(), button);
    if let Some(shortcut) = spec.shortcut {
        resp = resp.on_hover_text(format!("Shortcut: {shortcut}"));
    }
    if let Err(reason) = &enabled {
        resp = resp.on_disabled_hover_text(reason.to_string());
    }
    if resp.clicked() {
        actions.push(Action::new(id));
    }
    resp
}

/// A compact icon button for a registered command; the spec label becomes the tooltip.
pub fn command_icon(
    ui: &mut egui::Ui,
    engine: &Engine,
    id: &'static str,
    icon: &str,
    actions: &mut Vec<Action>,
) -> egui::Response {
    let Some(spec) = engine.registry().get(id) else {
        return ui.label(format!("unknown command {id}"));
    };
    let enabled = engine.enabled(id);
    let button = egui::Button::new(egui::RichText::new(icon).size(22.0)).frame(false);
    let tip = match spec.shortcut {
        Some(s) => format!("{} ({s})", spec.label),
        None => spec.label.to_owned(),
    };
    let resp = ui.add_enabled(enabled.is_ok(), button).on_hover_text(tip);
    if resp.clicked() {
        actions.push(Action::new(id));
    }
    resp
}
