//! pixelprobe desktop binary.
//!
//! `pixelprobe [--control PORT]` — with `--control`, a local JSON-lines control channel is
//! opened on `127.0.0.1:PORT` (see the `pp-control` crate).

#[cfg(not(target_arch = "wasm32"))]
mod displays;
#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    native::main()
}

// The web entry point is a separate gap (G12); the wasm target only has to compile.
#[cfg(target_arch = "wasm32")]
fn main() {}
