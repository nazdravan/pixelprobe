use crate::{Action, command_button, display_props};
use pp_engine::{Engine, json};

/// The main window: monitors, display properties and modes, and the test launchers.
pub fn show(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    egui::Panel::bottom("main_actions").show(ui, |ui| {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            command_button(ui, engine, "app.about", actions);
            crate::widgets::command_icon(ui, engine, "app.help", "?", actions);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // right_to_left: last added is leftmost.
                command_button(ui, engine, "test.pixel_refresh", actions);
                command_button(ui, engine, "test.pattern", actions);
                command_button(ui, engine, "test.motion", actions);
            });
        });
        ui.add_space(8.0);
    });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Available display monitors:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                command_button(ui, engine, "display.refresh", actions);
            });
        });
        monitor_list(ui, engine, actions);
        ui.add_space(12.0);
        ui.columns(2, |cols| {
            if let [left, right] = cols {
                left.label("Display properties:");
                properties(left, engine);
                right.label("Display modes:");
                modes(right, engine, actions);
            }
        });
    });
}

fn monitor_list(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    let displays = engine.displays();
    if displays.is_empty() {
        group(ui, 96.0, "No displays detected.");
        return;
    }
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::ScrollArea::vertical()
            .id_salt("pp-monitors")
            .max_height(96.0)
            .show(ui, |ui| {
                for (i, d) in displays.iter().enumerate() {
                    let selected = i == engine.doc().monitor;
                    let row = ui.selectable_label(selected, display_props::list_label(d));
                    if row.clicked() && !selected {
                        actions.push(Action {
                            id: "display.select",
                            params: json!({ "index": i }),
                        });
                    }
                }
            });
    });
}

/// The selected monitor's modes; the current one is marked, a click selects one.
fn modes(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    let Some(d) = engine.selected_display() else {
        group(ui, ui.available_height(), "Select a monitor.");
        return;
    };
    if d.modes.is_empty() {
        group(ui, ui.available_height(), "This monitor reports no modes.");
        return;
    }
    let current = d.current_mode();
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height(ui.available_height() - 16.0);
        egui::ScrollArea::vertical()
            .id_salt("pp-modes")
            .show(ui, |ui| {
                egui::Grid::new("pp-modes-grid")
                    .num_columns(4)
                    .striped(true)
                    .show(ui, |ui| {
                        for h in ["Resolution", "Bits", "Frequency", ""] {
                            ui.strong(h);
                        }
                        ui.end_row();
                        for (i, m) in d.modes.iter().enumerate() {
                            let text = format!("{} × {}", m.width, m.height);
                            let selected = engine.doc().mode == Some(i);
                            if ui.selectable_label(selected, text).clicked() && !selected {
                                actions.push(Action {
                                    id: "display.select_mode",
                                    params: json!({ "index": i }),
                                });
                            }
                            ui.label(if m.bits == 0 {
                                "—".to_owned()
                            } else {
                                format!("{} bit", m.bits)
                            });
                            ui.label(display_props::hz(m.refresh_hz()));
                            ui.weak(if current == Some(i) { "current" } else { "" });
                            ui.end_row();
                        }
                    });
            });
    });
}

fn properties(ui: &mut egui::Ui, engine: &Engine) {
    let Some(d) = engine.selected_display() else {
        group(ui, ui.available_height(), "Select a monitor.");
        return;
    };
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height(ui.available_height() - 16.0);
        egui::ScrollArea::vertical()
            .id_salt("pp-properties")
            .show(ui, |ui| {
                egui::Grid::new("pp-properties-grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        for (k, v) in display_props::properties(d) {
                            ui.label(k);
                            ui.label(v);
                            ui.end_row();
                        }
                    });
            });
    });
}

/// A framed box of at least `height` holding a muted placeholder text.
fn group(ui: &mut egui::Ui, height: f32, text: &str) {
    let frame = egui::Frame::group(ui.style());
    // The frame adds its margin and stroke around the content; keep the total within `height`.
    let chrome = frame.total_margin().sum().y;
    frame.show(ui, |ui| {
        ui.set_min_size(egui::vec2(
            ui.available_width(),
            (height - chrome).max(24.0),
        ));
        ui.weak(text);
    });
}
