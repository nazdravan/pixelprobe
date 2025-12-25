//! L5: session, command registry, undo history and panic guard.
//!
//! Every user-visible action is a [`CommandSpec`] in the [`Registry`]. UI code, the CLI and
//! the control channel all go through [`Engine::run`]; nothing else mutates the [`Doc`].

mod commands;
pub mod control;
mod error;
mod history;
mod registry;

pub use error::EngineError;
pub use history::History;
pub use pp_core::{self, DisplayInfo, Doc};
pub use registry::{CommandSpec, Registry, always_enabled};
pub use serde_json::{Value, json};

use std::panic::{AssertUnwindSafe, catch_unwind};

/// What a command sees: the document plus engine-owned state.
#[derive(Debug, Default)]
pub struct Session {
    pub doc: Doc,
    pub history: History,
    /// Displays as last reported by the platform (facts, not part of the document).
    pub displays: Vec<DisplayInfo>,
    /// Bumped by `display.refresh`; the app re-reads the displays when it changes.
    pub display_refreshes: u64,
}

/// Owns the session and the command registry.
pub struct Engine {
    session: Session,
    registry: Registry,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// An engine with all app commands registered.
    pub fn new() -> Self {
        Self::with_registry(Registry::app())
    }

    pub fn with_registry(registry: Registry) -> Self {
        Self {
            session: Session::default(),
            registry,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn doc(&self) -> &Doc {
        &self.session.doc
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Replace the display list with what the platform reports now. Not a command: these are
    /// facts about the machine, not a user change. Input is sanitized and bounded.
    pub fn set_displays(&mut self, displays: Vec<DisplayInfo>) {
        self.session.displays = displays
            .into_iter()
            .take(pp_core::display::MAX_DISPLAYS)
            .map(DisplayInfo::sanitized)
            .collect();
        let n = self.session.displays.len();
        if self.session.doc.monitor >= n {
            self.session.doc.monitor = 0;
            self.session.doc.mode = None;
        }
        let modes = self.selected_display().map_or(0, |d| d.modes.len());
        if self.session.doc.mode.is_some_and(|m| m >= modes) {
            self.session.doc.mode = None;
        }
    }

    pub fn displays(&self) -> &[DisplayInfo] {
        &self.session.displays
    }

    /// The display selected in the monitor list, if any.
    pub fn selected_display(&self) -> Option<&DisplayInfo> {
        self.session.displays.get(self.session.doc.monitor)
    }

    /// Whether `id` can run now; `Err(reason)` explains why not.
    pub fn enabled(&self, id: &str) -> Result<(), EngineError> {
        let spec = self
            .registry
            .get(id)
            .ok_or_else(|| EngineError::UnknownCommand(id.to_owned()))?;
        (spec.enabled)(&self.session).map_err(|reason| EngineError::Disabled {
            id: spec.id.to_owned(),
            reason,
        })
    }

    /// Run a command. On error or panic the document is rolled back to its prior state.
    /// A successful undoable command that changed the document is recorded in history.
    pub fn run(&mut self, id: &str, params: Value) -> Result<Value, EngineError> {
        self.enabled(id)?;
        let spec = self
            .registry
            .get(id)
            .ok_or_else(|| EngineError::UnknownCommand(id.to_owned()))?;
        let before = self.session.doc.clone();
        let run = spec.run;
        let outcome = catch_unwind(AssertUnwindSafe(|| run(&mut self.session, &params)));
        match outcome {
            Ok(Ok(value)) => {
                if spec.undoable && self.session.doc != before {
                    self.session.history.record(before);
                }
                Ok(value)
            }
            Ok(Err(err)) => {
                self.session.doc = before;
                Err(err)
            }
            Err(_) => {
                self.session.doc = before;
                Err(EngineError::Panicked(spec.id.to_owned()))
            }
        }
    }
}

/// Deserialize command params, mapping failures to [`EngineError::BadParams`].
pub fn params<T: serde::de::DeserializeOwned>(id: &str, value: &Value) -> Result<T, EngineError> {
    T::deserialize(value).map_err(|e| EngineError::BadParams {
        id: id.to_owned(),
        msg: e.to_string(),
    })
}

#[cfg(test)]
mod tests;
