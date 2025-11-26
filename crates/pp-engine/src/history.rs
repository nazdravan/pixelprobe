use pp_core::Doc;

/// Maximum number of undo steps kept.
pub const HISTORY_CAP: usize = 200;

/// Snapshot-based undo/redo. The document is small, so whole snapshots are cheap.
#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Doc>,
    redo: Vec<Doc>,
}

impl History {
    /// Record the state before a change; clears the redo stack.
    pub fn record(&mut self, before: Doc) {
        if self.undo.len() >= HISTORY_CAP {
            self.undo.remove(0);
        }
        self.undo.push(before);
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// Swap `current` with the previous state. Returns false if there is nothing to undo.
    pub fn undo(&mut self, current: &mut Doc) -> bool {
        match self.undo.pop() {
            Some(prev) => {
                self.redo.push(std::mem::replace(current, prev));
                true
            }
            None => false,
        }
    }

    /// Swap `current` with the next state. Returns false if there is nothing to redo.
    pub fn redo(&mut self, current: &mut Doc) -> bool {
        match self.redo.pop() {
            Some(next) => {
                self.undo.push(std::mem::replace(current, next));
                true
            }
            None => false,
        }
    }
}
