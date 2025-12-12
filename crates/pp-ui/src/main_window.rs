use crate::{Action, command_button};
use pp_engine::Engine;

/// The main window: monitors, display properties and modes, and the test launchers.
pub fn show(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    egui::Panel::bottom("main_actions").show(ui, |ui| {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            command_button(ui, engine, "app.about", actions);
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
        ui.label("Available display monitors:");
        group(ui, 96.0, "Monitor detection is not implemented yet (G8).");
        ui.add_space(12.0);
        ui.columns(2, |cols| {
            if let [left, right] = cols {
                left.label("Display properties:");
                group(left, left.available_height(), "Not available yet (G8).");
                right.label("Display modes:");
                group(right, right.available_height(), "Not available yet (G9).");
            }
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
