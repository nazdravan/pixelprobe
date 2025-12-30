//! About and help screens. The shortcut table is built from the command registry, so it
//! always matches what the keys really do.

use crate::Action;
use crate::widgets::command_button;
use pp_engine::Engine;

const TESTS: [(&str, &str); 4] = [
    (
        "Motion Test",
        "A shape moves across a full-screen background. Look for blur, trails (ghosting) and \
         uneven steps. Set the update rate close to the display refresh rate.",
    ),
    (
        "Pattern Test",
        "Full-screen patterns at the display's own resolution: SMPTE colour bars, solid \
         colours (dead and stuck pixels), gradients (banding), black and white level patches, \
         a grid (geometry) and checkerboards (sharpness).",
    ),
    (
        "Pixel Refresh",
        "Random red, green, blue, white and black cells change many times a second. Place the \
         noise over a stuck pixel and let it run; it may recover. Dead pixels cannot be fixed.",
    ),
    (
        "Display information",
        "The main window lists the monitors, their properties (size, pixels per inch, refresh \
         rate) and the display modes they support.",
    ),
];

/// (shortcut, what it does) for every command that has a shortcut, sorted by shortcut.
pub fn shortcut_rows(engine: &Engine) -> Vec<(&'static str, &'static str)> {
    let mut rows: Vec<_> = engine
        .registry()
        .iter()
        .filter_map(|c| Some((c.shortcut?, c.menu)))
        .collect();
    rows.sort_unstable();
    rows
}

pub fn help(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    egui::Panel::bottom("pp-help-buttons").show(ui, |ui| {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                command_button(ui, engine, "view.close", actions);
            });
        });
        ui.add_space(6.0);
    });
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Help");
            ui.add_space(6.0);
            for (title, text) in TESTS {
                ui.strong(title);
                ui.label(text);
                ui.add_space(6.0);
            }
            ui.add_space(6.0);
            ui.strong("Keyboard shortcuts");
            ui.label("Cmd is the Command key on macOS and Ctrl elsewhere. Keys work on the screen they belong to.");
            egui::Grid::new("pp-help-keys")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for (key, menu) in shortcut_rows(engine) {
                        ui.monospace(key);
                        ui.label(menu.replace('/', " › "));
                        ui.end_row();
                    }
                });
        });
    });
}

pub fn about(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            ui.heading("pixelprobe");
            ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
            ui.add_space(12.0);
            ui.label("A monitor test utility: display information, motion test,");
            ui.label("full-screen test patterns and pixel refresh.");
            ui.add_space(12.0);
            ui.label("Colour bars follow SMPTE RP 219:2002.");
            ui.hyperlink_to("Source code", "https://github.com/nazdravan/pixelprobe");
            ui.add_space(18.0);
            command_button(ui, engine, "view.close", actions);
        });
    });
}
