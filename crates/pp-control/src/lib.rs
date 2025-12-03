//! L6: local control channel. Lets scripts and agents run commands and take screenshots of
//! the running app: `pixelprobe --control 7878`, then send JSON lines to `127.0.0.1:7878`.
//!
//! Protocol: see [`pp_engine::control`]. Extra UI-level methods handled here:
//! - `app.commands` → list of command specs
//! - `app.state` → the current document
//! - `ui.screenshot {"path": "out.png"}` → PNG of the next painted frame
//!
//! Native only (no sockets in the browser).

mod dispatch;
mod server;

pub use dispatch::handle;
pub use server::{ControlServer, Pending};

/// Errors starting the control server.
#[derive(Debug, thiserror::Error)]
pub enum ControlError {
    #[error("cannot listen on 127.0.0.1:{port}: {source}")]
    Bind { port: u16, source: std::io::Error },
    #[error("cannot start control thread: {0}")]
    Thread(std::io::Error),
}
