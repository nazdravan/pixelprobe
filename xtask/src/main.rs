//! Project automation: `cargo xtask <ci|fmt|clippy|test|layers|wasm|web>`.

use std::collections::BTreeMap;
use std::process::{Command, ExitCode};

/// Crates that must never be depended on below this layer.
const UI_LAYER: u64 = 6;
const UI_ONLY_DEPS: &[&str] = &["egui", "eframe", "winit", "rfd", "egui-winit", "egui-wgpu"];
/// Crates checked for `wasm32-unknown-unknown`.
const WASM_CRATES: &[&str] = &["pp-core", "pp-patterns", "pp-engine", "pp-ui", "pixelprobe"];

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_else(|| "help".into());
    let result = match task.as_str() {
        "ci" => ci(),
        "fmt" => fmt(),
        "clippy" => clippy(),
        "test" => test(),
        "layers" => layers(),
        "wasm" => wasm(),
        "web" => web(),
        _ => {
            eprintln!("usage: cargo xtask <ci|fmt|clippy|test|layers|wasm|web>");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("xtask {task}: FAILED — {msg}");
            ExitCode::FAILURE
        }
    }
}

type Res = Result<(), String>;

fn ci() -> Res {
    for (name, step) in [
        ("fmt", fmt as fn() -> Res),
        ("clippy", clippy),
        ("test", test),
        ("layers", layers),
        ("wasm", wasm),
    ] {
        eprintln!("== xtask ci: {name}");
        step()?;
    }
    eprintln!("== xtask ci: all green");
    Ok(())
}

fn cargo(args: &[&str]) -> Res {
    let mut cmd = Command::new(env_cargo());
    cmd.args(args);
    // When several Rust installs are on PATH (e.g. Homebrew + rustup), cargo may pick
    // rustc/rustdoc/cargo-clippy from another install (no wasm std, mismatched metadata).
    // Put the directory of the running cargo first so every tool comes from one toolchain.
    if let Some(dir) = std::path::Path::new(&env_cargo()).parent()
        && dir
            .join(format!("rustc{}", std::env::consts::EXE_SUFFIX))
            .is_file()
    {
        let mut paths = vec![dir.to_path_buf()];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        if let Ok(joined) = std::env::join_paths(paths) {
            cmd.env("PATH", joined);
        }
    }
    let status = cmd.status().map_err(|e| format!("cannot run cargo: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo {}` exited with {status}", args.join(" ")))
    }
}

fn env_cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".into())
}

fn fmt() -> Res {
    cargo(&["fmt", "--all", "--check"])
}

fn clippy() -> Res {
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])
}

fn test() -> Res {
    cargo(&["test", "--workspace", "--quiet"])
}

fn wasm() -> Res {
    let mut args = vec!["check", "--target", "wasm32-unknown-unknown"];
    for krate in WASM_CRATES {
        args.push("-p");
        args.push(krate);
    }
    cargo(&args)
}

/// Where `cargo xtask web` puts the site.
const WEB_OUT: &str = "target/web";

/// Build the web app into `target/web` (index.html + pixelprobe.js + wasm). Needs the
/// `wasm-bindgen` CLI at the same version as the `wasm-bindgen` crate in Cargo.lock.
fn web() -> Res {
    cargo(&[
        "build",
        "--release",
        "--target",
        "wasm32-unknown-unknown",
        "-p",
        "pixelprobe",
    ])?;
    let wasm = "target/wasm32-unknown-unknown/release/pixelprobe.wasm";
    let status = Command::new("wasm-bindgen")
        .args([
            "--target",
            "web",
            "--no-typescript",
            "--out-name",
            "pixelprobe",
            "--out-dir",
            WEB_OUT,
            wasm,
        ])
        .status()
        .map_err(|e| format!("cannot run wasm-bindgen (cargo install wasm-bindgen-cli): {e}"))?;
    if !status.success() {
        return Err(format!("wasm-bindgen exited with {status}"));
    }
    std::fs::copy("app/web/index.html", format!("{WEB_OUT}/index.html"))
        .map_err(|e| format!("cannot copy index.html: {e}"))?;
    std::fs::copy("assets/icon.png", format!("{WEB_OUT}/icon.png"))
        .map_err(|e| format!("cannot copy icon.png: {e}"))?;
    eprintln!("web build in {WEB_OUT}; serve it, e.g. python3 -m http.server -d {WEB_OUT}");
    Ok(())
}

/// Every workspace crate declares `[package.metadata.pixelprobe] layer = N`; a crate may
/// depend only on workspace crates of a lower layer, and nothing below L6 may pull in UI crates.
fn layers() -> Res {
    let out = Command::new(env_cargo())
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("cannot run cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err("cargo metadata failed".into());
    }
    let meta: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("bad metadata: {e}"))?;
    let packages = meta
        .get("packages")
        .and_then(|p| p.as_array())
        .ok_or("metadata has no packages")?;

    let mut layer_of = BTreeMap::new();
    for pkg in packages {
        let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        let layer = pkg
            .pointer("/metadata/pixelprobe/layer")
            .and_then(|l| l.as_u64())
            .ok_or_else(|| format!("{name}: missing [package.metadata.pixelprobe] layer"))?;
        layer_of.insert(name.to_owned(), layer);
    }

    let mut errors = Vec::new();
    for pkg in packages {
        let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        let layer = layer_of.get(name).copied().unwrap_or(0);
        let deps = pkg.get("dependencies").and_then(|d| d.as_array());
        for dep in deps.into_iter().flatten() {
            if dep.get("kind").and_then(|k| k.as_str()) == Some("dev") {
                continue;
            }
            let dep_name = dep.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            if let Some(&dep_layer) = layer_of.get(dep_name)
                && dep_layer >= layer
            {
                errors.push(format!(
                    "{name} (L{layer}) depends on {dep_name} (L{dep_layer})"
                ));
            }
            if layer < UI_LAYER && UI_ONLY_DEPS.contains(&dep_name) {
                errors.push(format!("{name} (L{layer}) depends on UI crate {dep_name}"));
            }
        }
    }
    if errors.is_empty() {
        eprintln!("layers ok ({} crates)", layer_of.len());
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
