//! L6: egui panels.
//!
//! Panels only *read* the engine and return the [`Action`]s the user asked for; the app runs
//! them through `Engine::run`. Nothing here mutates the document.

pub mod display_props;
mod main_window;
mod motion_view;
mod pattern_view;
pub(crate) mod pixel_refresh;
pub mod shortcuts;
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
    let mut actions = shortcuts::actions(ui.ctx(), engine);
    match engine.doc().screen {
        Screen::Main => main_window::show(ui, engine, &mut actions),
        Screen::MotionTest => motion_view::show(ui, engine, &mut actions),
        Screen::PatternTest => pattern_view::show(ui, engine, &mut actions),
        Screen::PixelRefresh => pixel_refresh::show(ui, engine, &mut actions),
        Screen::About => about(ui, engine, &mut actions),
    }
    actions
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
