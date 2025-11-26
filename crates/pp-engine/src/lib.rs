//! L5: session, command registry, undo history and panic guard.
//!
//! Every user-visible action is a [`CommandSpec`] in the [`Registry`]. UI code, the CLI and
//! the control channel all go through [`Engine::run`]; nothing else mutates the [`Doc`].

mod error;
mod history;
mod registry;

pub use error::EngineError;
pub use history::History;
pub use pp_core::{self, Doc};
pub use registry::{CommandSpec, Registry, always_enabled};
pub use serde_json::{Value, json};

use std::panic::{AssertUnwindSafe, catch_unwind};

/// What a command sees: the document plus engine-owned state.
#[derive(Debug, Default)]
pub struct Session {
    pub doc: Doc,
    pub history: History,
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
    /// An engine with the built-in commands registered.
    pub fn new() -> Self {
        Self::with_registry(Registry::builtin())
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
