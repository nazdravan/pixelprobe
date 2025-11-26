/// Why a command did not run or failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EngineError {
    #[error("unknown command `{0}`")]
    UnknownCommand(String),
    #[error("`{id}` is disabled: {reason}")]
    Disabled { id: String, reason: String },
    #[error("bad params for `{id}`: {msg}")]
    BadParams { id: String, msg: String },
    #[error("`{id}` failed: {msg}")]
    Failed { id: String, msg: String },
    #[error("`{0}` panicked; the document was rolled back")]
    Panicked(String),
}
