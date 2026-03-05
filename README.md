# pixelprobe

A monitor test utility written in Rust (egui/eframe, desktop + web).

- **Display information**: the monitors, their properties (resolution, physical size,
  pixels per inch, diagonal, aspect ratios, refresh rate, colour depth) and display modes.
- **Pattern test**: full-screen patterns at the display's own resolution: SMPTE RP 219
  colour bars (with the standard's *2/*3 sub-pattern options), solid colours, gradients,
  black/white level patches, grid, checkerboards. Optionally in a chosen display mode.
- **Motion test**: a moving square or circle with adjustable update rate, size, colours and
  step, for blur and ghosting.
- **Pixel refresh**: random red/green/blue/white/black noise to exercise stuck pixels, in a
  window or full screen.

Press F1 in the app for help and the list of keyboard shortcuts.

## Build

```sh
cargo run -p pixelprobe      # desktop app
cargo xtask ci               # fmt, clippy -D warnings, tests, layer rules, wasm check
cargo xtask web              # web build into target/web (needs wasm-bindgen-cli)
python3 -m http.server -d target/web   # then open http://localhost:8000
```

Requires a rustup toolchain (see `rust-toolchain.toml`; the wasm target is installed
automatically). The `wasm-bindgen` CLI must match the `wasm-bindgen` version in `Cargo.lock`.

In the browser, display information is limited to what `window.screen` reports (no refresh
rate, physical size or modes), and full screen uses the browser's Fullscreen API.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.
