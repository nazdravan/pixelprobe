//! Pixel refresh: random red/green/blue/white/black cells that flip at the chosen rate, to
//! exercise a stuck subpixel, with the speed, update rate and elapsed time.

use crate::Action;
use crate::widgets::{command_button, opened_at};
use pp_engine::pp_core::RefreshSpeed;
use pp_engine::{Engine, json};
use pp_patterns::noise;

/// Side of one noise cell, in points.
const CELL_POINTS: f32 = 8.0;
/// Height kept for everything below the noise, in points.
const CONTROLS_HEIGHT: f32 = 250.0;

/// The noise texture on screen and the frame it shows.
#[derive(Clone)]
struct Cached {
    key: (u64, u32, u32),
    texture: egui::TextureHandle,
}

pub fn show(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    let speed = engine.doc().refresh_speed;
    let now = ui.input(|i| i.time);
    let elapsed = (now - opened_at(ui, "pp-refresh-opened")).max(0.0);
    let rate = f64::from(speed.rate_hz());
    let frame = (elapsed * rate).floor();
    // Wake up for the next noise frame (and so the elapsed time ticks).
    ui.ctx()
        .request_repaint_after_secs(((frame + 1.0) / rate - elapsed) as f32);

    egui::Panel::bottom("pp-refresh-buttons").show(ui, |ui| {
        ui.add_space(6.0);
        // A row first: right_to_left on its own would claim the whole panel height.
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                command_button(ui, engine, "view.close", actions);
            });
        });
        ui.add_space(6.0);
    });
    egui::CentralPanel::default().show(ui, |ui| {
        let noise_height = (ui.available_height() - CONTROLS_HEIGHT).max(96.0);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), noise_height),
            egui::Sense::hover(),
        );
        paint_noise(ui, rect, frame as u64);
        ui.add_space(10.0);
        controls(ui, speed, elapsed, actions);
        ui.add_space(10.0);
        ui.label("Pixel refresh flashes every subpixel fully on and off. It can bring a stuck pixel back, but there is no guarantee.");
        ui.add_space(6.0);
        ui.label("Move this window so the stuck pixel lies inside the noise and let it run for 10–20 minutes.");
        ui.add_space(6.0);
        ui.label("A dead pixel (always black) cannot be fixed this way.");
    });
}

fn controls(ui: &mut egui::Ui, speed: RefreshSpeed, elapsed: f64, actions: &mut Vec<Action>) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let label = ui.label("Speed:");
            ui.spacing_mut().slider_width = ui.available_width() - 8.0;
            let mut v = speed.get();
            let slider =
                egui::Slider::new(&mut v, RefreshSpeed::MIN..=RefreshSpeed::MAX).show_value(false);
            let resp = ui.add(slider).labelled_by(label.id);
            if resp.changed() && v != speed.get() {
                actions.push(Action {
                    id: "refresh.speed",
                    params: json!({ "speed": v }),
                });
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(format!("Update rate: {} /s", speed.rate_hz()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Elapsed time: {}", clock(elapsed)));
            });
        });
    });
}

/// `mm:ss`, or `h:mm:ss` from one hour on.
pub fn clock(secs: f64) -> String {
    let s = if secs.is_finite() && secs > 0.0 {
        secs.floor().min(f64::from(u32::MAX)) as u64
    } else {
        0
    };
    let (h, m, s) = (s / 3600, s / 60 % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

/// Fill `rect` with whole noise cells aligned to physical pixels, centred.
fn paint_noise(ui: &mut egui::Ui, rect: egui::Rect, frame: u64) {
    let ppp = ui.ctx().pixels_per_point();
    let cell_px = (CELL_POINTS * ppp).round().max(1.0);
    let cells = |len: f32| {
        let n = (len * ppp / cell_px).floor();
        if n.is_finite() && n >= 1.0 {
            n.min(4096.0) as u32
        } else {
            1
        }
    };
    let (cols, rows) = (cells(rect.width()), cells(rect.height()));
    let size = egui::vec2(cols as f32, rows as f32) * cell_px / ppp;
    let painted = egui::Rect::from_center_size(rect.center(), size);
    // Snap the corner to a physical pixel so every cell is exactly `cell_px` wide.
    let min = (painted.min.to_vec2() * ppp).round() / ppp;
    let painted = egui::Rect::from_min_size(min.to_pos2(), size);

    let key = (frame, cols, rows);
    let id = egui::Id::unique("pp-refresh-texture");
    let cached = ui.ctx().data(|d| d.get_temp::<Cached>(id));
    let texture = match cached {
        Some(c) if c.key == key => c.texture,
        cached => {
            let image = match noise(cols, rows, frame) {
                Ok(img) => img,
                Err(e) => {
                    ui.colored_label(egui::Color32::LIGHT_RED, e.to_string());
                    return;
                }
            };
            let size = [image.width as usize, image.height as usize];
            let color = egui::ColorImage::from_rgba_unmultiplied(size, &image.rgba);
            let texture = match cached {
                Some(mut c) => {
                    c.texture.set(color, egui::TextureOptions::NEAREST);
                    c.texture
                }
                None => ui
                    .ctx()
                    .load_texture("pp-refresh", color, egui::TextureOptions::NEAREST),
            };
            ui.ctx().data_mut(|d| {
                d.insert_temp(
                    id,
                    Cached {
                        key,
                        texture: texture.clone(),
                    },
                );
            });
            texture
        }
    };
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    ui.painter()
        .image(texture.id(), painted, uv, egui::Color32::WHITE);
}
