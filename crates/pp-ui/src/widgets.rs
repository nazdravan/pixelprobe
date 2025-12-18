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

/// When the screen drawn under `key` was opened, in `ui.input(|i| i.time)` seconds.
///
/// A gap in drawn passes means the screen was off and has just been opened again.
pub fn opened_at(ui: &egui::Ui, key: &str) -> f64 {
    let ctx = ui.ctx();
    let id = egui::Id::unique(key);
    let (now, pass) = (ui.input(|i| i.time), ctx.cumulative_pass_nr());
    let opened = match ctx.data(|d| d.get_temp::<(f64, u64)>(id)) {
        Some((t, last)) if last + 1 >= pass => t,
        _ => now,
    };
    ctx.data_mut(|d| d.insert_temp(id, (opened, pass)));
    opened
}
