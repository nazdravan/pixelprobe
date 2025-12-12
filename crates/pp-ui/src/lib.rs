//! L6: egui panels.
//!
//! Panels only *read* the engine and return the [`Action`]s the user asked for; the app runs
//! them through `Engine::run`. Nothing here mutates the document.

mod main_window;
mod widgets;

pub use egui;
pub use pp_engine;
pub use widgets::command_button;

use pp_engine::pp_core::Screen;
use pp_engine::{Engine, Value};

/// A command the user triggered this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub id: &'static str,
    pub params: Value,
}

impl Action {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            params: Value::Null,
        }
    }
}

/// Draw the whole app for the current screen; returns what the user triggered.
pub fn show(ui: &mut egui::Ui, engine: &Engine) -> Vec<Action> {
    let mut actions = Vec::new();
    match engine.doc().screen {
        Screen::Main => main_window::show(ui, engine, &mut actions),
        Screen::MotionTest => placeholder(ui, engine, "Motion Test", "G6", &mut actions),
        Screen::PatternTest => placeholder(ui, engine, "Pattern Test", "G5", &mut actions),
        Screen::PixelRefresh => placeholder(ui, engine, "Pixel Refresh", "G7", &mut actions),
        Screen::About => about(ui, engine, &mut actions),
    }
    actions
}

/// Screens whose feature has not landed yet: say so plainly and offer the way back.
fn placeholder(
    ui: &mut egui::Ui,
    engine: &Engine,
    title: &str,
    gap: &str,
    actions: &mut Vec<Action>,
) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading(title);
        ui.add_space(8.0);
        ui.label(format!("This test is not implemented yet ({gap})."));
        ui.add_space(8.0);
        command_button(ui, engine, "view.close", actions);
    });
}

fn about(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("pixelprobe");
        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
        ui.add_space(8.0);
        ui.label("Monitor test utility: display information, motion test,");
        ui.label("full-screen test patterns and pixel refresh.");
        ui.add_space(8.0);
        command_button(ui, engine, "view.close", actions);
    });
}

#[cfg(test)]
mod tests;
