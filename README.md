# pixelprobe

A monitor test utility written in Rust (egui/eframe, native + web):
display information, motion test, full-screen test patterns and pixel refresh.

Status: early bootstrap — no user-facing features yet.

## Build

```sh
cargo run -p pixelprobe      # desktop app
cargo xtask ci               # fmt, clippy -D warnings, tests, layer rules, wasm check
```

Requires a rustup toolchain (see `rust-toolchain.toml`; the wasm target is installed automatically).
