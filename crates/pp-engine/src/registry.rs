use crate::{EngineError, Session, Value};
use std::collections::BTreeMap;

/// One user-visible action.
#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    /// Stable id, e.g. `pattern.next`.
    pub id: &'static str,
    /// Human label for buttons and menus.
    pub label: &'static str,
    /// Menu path, e.g. `Edit/Undo`; empty if not in a menu.
    pub menu: &'static str,
    /// Platform-neutral shortcut, e.g. `Cmd+Z`, `Right`, `Escape`. `Cmd` means Ctrl on
    /// Windows/Linux. The UI layer maps it to its key events.
    pub shortcut: Option<&'static str>,
    /// Documentation of the JSON params object (empty if the command takes none).
    pub params: &'static str,
    /// Whether a successful document change goes into undo history.
    pub undoable: bool,
    pub enabled: fn(&Session) -> Result<(), String>,
    pub run: fn(&mut Session, &Value) -> Result<Value, EngineError>,
}

/// Always enabled.
pub fn always_enabled(_: &Session) -> Result<(), String> {
    Ok(())
}

/// All registered commands, by id.
#[derive(Debug, Default)]
pub struct Registry {
    commands: BTreeMap<&'static str, CommandSpec>,
}

impl Registry {
    pub fn empty() -> Self {
        Self::default()
    }

    /// The commands every engine has (undo/redo).
    pub fn builtin() -> Self {
        let mut r = Self::empty();
        r.register(CommandSpec {
            id: "edit.undo",
            label: "Undo",
            menu: "Edit/Undo",
            shortcut: Some("Cmd+Z"),
            params: "",
            undoable: false,
            enabled: |s| {
                if s.history.can_undo() {
                    Ok(())
                } else {
                    Err("nothing to undo".into())
                }
            },
            run: |s, _| {
                // Navigation and selection are not history: keep the screen and monitor.
                let Session { doc, history, .. } = s;
                let (screen, monitor) = (doc.screen, doc.monitor);
                history.undo(doc);
                (doc.screen, doc.monitor) = (screen, monitor);
                Ok(Value::Null)
            },
        });
        r.register(CommandSpec {
            id: "edit.redo",
            label: "Redo",
            menu: "Edit/Redo",
            shortcut: Some("Cmd+Shift+Z"),
            params: "",
            undoable: false,
            enabled: |s| {
                if s.history.can_redo() {
                    Ok(())
                } else {
                    Err("nothing to redo".into())
                }
            },
            run: |s, _| {
                // Navigation and selection are not history: keep the screen and monitor.
                let Session { doc, history, .. } = s;
                let (screen, monitor) = (doc.screen, doc.monitor);
                history.redo(doc);
                (doc.screen, doc.monitor) = (screen, monitor);
                Ok(Value::Null)
            },
        });
        r
    }

    /// Built-ins plus every app command.
    pub fn app() -> Self {
        let mut r = Self::builtin();
        crate::commands::register_app(&mut r);
        r
    }

    /// Add or replace a command.
    pub fn register(&mut self, spec: CommandSpec) {
        self.commands.insert(spec.id, spec);
    }

    pub fn get(&self, id: &str) -> Option<&CommandSpec> {
        self.commands.get(id)
    }

    /// All commands, sorted by id.
    pub fn iter(&self) -> impl Iterator<Item = &CommandSpec> {
        self.commands.values()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}
