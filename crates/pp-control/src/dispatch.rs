use crate::Pending;
use pp_engine::{Engine, Value, json};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ScreenshotParams {
    path: PathBuf,
}

/// Answer one control request on the UI thread.
pub fn handle(engine: &mut Engine, ctx: &egui::Context, pending: Pending) {
    match pending.request.method.as_str() {
        "app.commands" => {
            let list: Vec<Value> = engine
                .registry()
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "label": c.label,
                        "menu": c.menu,
                        "shortcut": c.shortcut,
                        "params": c.params,
                        "enabled": engine.enabled(c.id).is_ok(),
                    })
                })
                .collect();
            pending.reply(Ok(Value::Array(list)));
        }
        "app.state" => {
            let state = serde_json::to_value(engine.doc()).map_err(|e| e.to_string());
            pending.reply(state);
        }
        "ui.screenshot" => screenshot(ctx, pending),
        _ => {
            let (request, replier) = pending.into_parts();
            let result = engine
                .run(&request.method, request.params)
                .map_err(|e| e.to_string());
            ctx.request_repaint();
            replier.reply(result);
        }
    }
}

fn screenshot(ctx: &egui::Context, pending: Pending) {
    let (request, replier) = pending.into_parts();
    let params: ScreenshotParams =
        match pp_engine::params("ui.screenshot", &request.params).map_err(|e| e.to_string()) {
            Ok(p) => p,
            Err(e) => return replier.reply(Err(e)),
        };
    if params.path.extension().and_then(|e| e.to_str()) != Some("png") {
        return replier.reply(Err("ui.screenshot: path must end in .png".into()));
    }
    ctx.request_screenshot(move |image: Arc<egui::ColorImage>| {
        let result = write_png(&params.path, &image).map(|()| {
            json!({
                "path": params.path.display().to_string(),
                "width": image.size[0],
                "height": image.size[1],
            })
        });
        replier.reply(result);
    });
    ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
    ctx.request_repaint();
}

/// Encode an egui image as an RGBA PNG.
pub(crate) fn write_png(path: &std::path::Path, image: &egui::ColorImage) -> Result<(), String> {
    let [w, h] = image.size;
    let (Ok(w32), Ok(h32)) = (u32::try_from(w), u32::try_from(h)) else {
        return Err("image too large".into());
    };
    if image.pixels.len() != w.saturating_mul(h) || w == 0 || h == 0 {
        return Err("empty or inconsistent image".into());
    }
    let rgba: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|c| c.to_srgba_unmultiplied())
        .collect();
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w32, h32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&rgba).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_round_trip_size() {
        let dir = std::env::temp_dir().join(format!("pp-control-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.png");
        let img = egui::ColorImage::filled([3, 2], egui::Color32::RED);
        write_png(&path, &img).unwrap();
        let decoder =
            png::Decoder::new(std::io::BufReader::new(std::fs::File::open(&path).unwrap()));
        let reader = decoder.read_info().unwrap();
        assert_eq!((reader.info().width, reader.info().height), (3, 2));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn png_rejects_empty() {
        let img = egui::ColorImage::filled([0, 0], egui::Color32::RED);
        assert!(write_png(std::path::Path::new("/nonexistent/x.png"), &img).is_err());
    }
}
