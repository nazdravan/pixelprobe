//! Full-screen pattern test: the pattern at physical resolution plus a small toolbar.

use crate::Action;
use crate::widgets::{command_icon, opened_at};
use pp_engine::Engine;
use pp_engine::pp_core::PatternId;
use pp_patterns::render;

/// Seconds without pointer movement before the toolbar hides.
const TOOLBAR_IDLE_SECS: f32 = 3.0;

/// Cached texture for the pattern currently on screen.
#[derive(Clone)]
struct Cached {
    key: (PatternId, u32, u32),
    texture: egui::TextureHandle,
}

pub fn show(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    let pattern = engine.doc().pattern;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
        .show(ui, |ui| {
            let rect = ui.max_rect();
            paint_pattern(ui, pattern, rect);
            if toolbar_visible(ui) {
                toolbar(ui, engine, pattern, rect, actions);
            }
        });
}

/// The toolbar shows for a few seconds after the test opens or the pointer moves, and
/// whenever the pointer hovers the top edge.
fn toolbar_visible(ui: &egui::Ui) -> bool {
    let ctx = ui.ctx();
    let now = ui.input(|i| i.time);
    let opened = opened_at(ui, "pp-pattern-opened");
    let since_open = (now - opened) as f32;
    let idle = ui.input(|i| i.pointer.time_since_last_movement());
    let recent = since_open.min(if idle.is_finite() { idle } else { f32::MAX });
    if recent < TOOLBAR_IDLE_SECS {
        // Repaint once more when it is time to hide.
        ctx.request_repaint_after_secs(TOOLBAR_IDLE_SECS - recent + 0.05);
        return true;
    }
    ui.input(|i| i.pointer.hover_pos())
        .is_some_and(|p| p.y < 64.0)
}

/// Render (or reuse) the pattern at the rect's physical pixel size and paint it 1:1.
fn paint_pattern(ui: &mut egui::Ui, pattern: PatternId, rect: egui::Rect) {
    let ppp = ui.ctx().pixels_per_point();
    let to_px = |v: f32| {
        let p = (v * ppp).round();
        if p.is_finite() && p >= 1.0 {
            p as u32
        } else {
            1
        }
    };
    let key = (pattern, to_px(rect.width()), to_px(rect.height()));
    let id = egui::Id::unique("pp-pattern-texture");
    let cached = ui.ctx().data(|d| d.get_temp::<Cached>(id));
    let texture = match cached {
        Some(c) if c.key == key => c.texture,
        _ => {
            let image = match render(key.0, key.1, key.2) {
                Ok(img) => img,
                Err(e) => {
                    ui.colored_label(egui::Color32::LIGHT_RED, e.to_string());
                    return;
                }
            };
            let size = [image.width as usize, image.height as usize];
            if image.rgba.len() != size[0].saturating_mul(size[1]).saturating_mul(4) {
                ui.colored_label(egui::Color32::LIGHT_RED, "pattern size mismatch");
                return;
            }
            let color = egui::ColorImage::from_rgba_unmultiplied(size, &image.rgba);
            let texture = ui
                .ctx()
                .load_texture("pp-pattern", color, egui::TextureOptions::NEAREST);
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
        .image(texture.id(), rect, uv, egui::Color32::WHITE);
}

fn toolbar(
    ui: &mut egui::Ui,
    engine: &Engine,
    pattern: PatternId,
    screen: egui::Rect,
    actions: &mut Vec<Action>,
) {
    egui::Area::new(egui::Id::unique("pp-pattern-toolbar"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 0.0))
        .constrain_to(screen)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .corner_radius(0.0)
                .inner_margin(egui::Margin::symmetric(16, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [160.0, 24.0],
                            egui::Label::new(egui::RichText::new(pattern.name()).size(15.0)),
                        );
                        command_icon(ui, engine, "pattern.prev", "‹", actions);
                        command_icon(ui, engine, "pattern.next", "›", actions);
                        ui.separator();
                        command_icon(ui, engine, "view.close", "×", actions);
                    });
                });
        });
}
