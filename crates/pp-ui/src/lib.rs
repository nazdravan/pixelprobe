//! L6: egui panels.
//!
//! Panels only *read* the engine and return the [`Action`]s the user asked for; the app runs
//! them through `Engine::run`. Nothing here mutates the document.

pub mod display_props;
pub(crate) mod help;
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
        Screen::About => help::about(ui, engine, &mut actions),
        Screen::Help => help::help(ui, engine, &mut actions),
    }
    actions
}

#[cfg(test)]
mod tests;
