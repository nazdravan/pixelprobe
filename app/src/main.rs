//! pixelprobe desktop and web binary.
//!
//! Desktop: `pixelprobe [--control PORT]` — with `--control`, a local JSON-lines control
//! channel is opened on `127.0.0.1:PORT` (see the `pp-control` crate).
//! Web: built for `wasm32-unknown-unknown`, it starts on the page's `pixelprobe` canvas
//! (`cargo xtask web` assembles the site).

mod shell;

#[cfg(not(target_arch = "wasm32"))]
mod displays;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    native::main()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    web::main();
}
