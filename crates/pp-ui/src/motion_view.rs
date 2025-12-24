//! Full-screen motion test: a shape that moves across the screen at a chosen update rate,
//! with the speed readouts and the shape, size, colour and step options on top.

use crate::Action;
use crate::widgets::{command_button, opened_at};
use pp_engine::pp_core::{Motion, MotionSpeed, MoveStep, Rgb8, Shape, ShapeSize};
use pp_engine::{Engine, Value, json};
use pp_patterns::motion_x;

/// How long the render rate is averaged over, in seconds.
const RENDER_WINDOW: f64 = 1.0;

pub fn show(ui: &mut egui::Ui, engine: &Engine, actions: &mut Vec<Action>) {
    let m = engine.doc().motion;
    let now = ui.input(|i| i.time);
    let elapsed = (now - opened_at(ui, "pp-motion-opened")).max(0.0);
    let render_rate = render_rate(ui, now);
    // Animate continuously: each display refresh draws a frame, so the render rate shows
    // the display's real rate.
    ui.ctx().request_repaint();

    egui::Panel::top("pp-motion-controls").show(ui, |ui| {
        ui.add_space(4.0);
        ui.horizontal_top(|ui| {
            speed_group(ui, engine, &m, render_rate, actions);
            options_group(ui, engine, &m, actions);
        });
        ui.add_space(4.0);
    });
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(color(m.background)))
        .show(ui, |ui| {
            let rect = ui.max_rect();
            paint_shape(ui, &m, rect, elapsed);
        });
}

fn color(c: Rgb8) -> egui::Color32 {
    egui::Color32::from_rgb(c.r, c.g, c.b)
}

/// Frames drawn per second over the last [`RENDER_WINDOW`].
fn render_rate(ui: &egui::Ui, now: f64) -> f64 {
    let id = egui::Id::unique("pp-motion-frames");
    let mut times: Vec<f64> = ui.ctx().data(|d| d.get_temp(id)).unwrap_or_default();
    times.retain(|t| now - t < RENDER_WINDOW && *t <= now);
    times.push(now);
    // Bounded: at most one entry per frame within the window; cap anyway.
    if times.len() > 4096 {
        let extra = times.len() - 4096;
        times.drain(..extra);
    }
    let rate = match (times.first(), times.last()) {
        (Some(a), Some(b)) if b > a => (times.len() - 1) as f64 / (b - a),
        _ => 0.0,
    };
    ui.ctx().data_mut(|d| d.insert_temp(id, times));
    rate
}

fn set(actions: &mut Vec<Action>, params: Value) {
    actions.push(Action {
        id: "motion.set",
        params,
    });
}

fn speed_group(
    ui: &mut egui::Ui,
    engine: &Engine,
    m: &Motion,
    render_rate: f64,
    actions: &mut Vec<Action>,
) {
    ui.group(|ui| {
        ui.set_width(380.0);
        ui.vertical(|ui| {
            ui.label("Speed");
            ui.horizontal(|ui| {
                command_button_small(ui, engine, "motion.slower", actions);
                ui.spacing_mut().slider_width = 270.0;
                let mut v = m.speed.get();
                let slider = egui::Slider::new(&mut v, MotionSpeed::MIN..=MotionSpeed::MAX)
                    .show_value(false);
                if ui.add(slider).changed() && v != m.speed.get() {
                    set(actions, json!({ "speed": v }));
                }
                command_button_small(ui, engine, "motion.faster", actions);
            });
            let rate = m.speed.rate_hz();
            egui::Grid::new("pp-motion-rates")
                .num_columns(4)
                .show(ui, |ui| {
                    ui.label("Update interval:");
                    ui.label(format!("{:.1} ms", 1000.0 / f64::from(rate.max(1))));
                    ui.label("Refresh rate:");
                    ui.label(refresh_text(engine));
                    ui.end_row();
                    ui.label("Update rate:");
                    ui.label(format!("{rate} /s"));
                    ui.label("Render rate:");
                    ui.label(format!("{render_rate:.0} /s"));
                    ui.end_row();
                });
            ui.label("Set the update rate close to the refresh rate for the clearest motion.");
        });
    });
}

/// The refresh rate of the display showing the window (else the selected one).
fn refresh_text(engine: &Engine) -> String {
    let shown = engine.displays().iter().find(|d| d.has_window);
    crate::display_props::refresh_text(shown.or_else(|| engine.selected_display()))
}

/// A small `-`/`+` button for a command; its label comes from the spec.
fn command_button_small(
    ui: &mut egui::Ui,
    engine: &Engine,
    id: &'static str,
    actions: &mut Vec<Action>,
) {
    let Some(spec) = engine.registry().get(id) else {
        return;
    };
    let enabled = engine.enabled(id);
    let tip = spec
        .shortcut
        .map_or(spec.menu.to_owned(), |k| format!("{} ({k})", spec.menu));
    let resp = ui
        .add_enabled(
            enabled.is_ok(),
            egui::Button::new(spec.label).min_size(egui::vec2(24.0, 20.0)),
        )
        .on_hover_text(tip);
    if resp.clicked() {
        actions.push(Action::new(id));
    }
}

fn options_group(ui: &mut egui::Ui, engine: &Engine, m: &Motion, actions: &mut Vec<Action>) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label("Options");
            egui::Grid::new("pp-motion-options")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Shape:");
                    ui.horizontal(|ui| {
                        for (v, name) in [(Shape::Square, "Square"), (Shape::Circle, "Circle")] {
                            if ui.radio(m.shape == v, name).clicked() && m.shape != v {
                                set(actions, json!({ "shape": v }));
                            }
                        }
                        let mut hollow = m.hollow;
                        if ui.checkbox(&mut hollow, "Hollow").changed() {
                            set(actions, json!({ "hollow": hollow }));
                        }
                    });
                    ui.end_row();

                    ui.label("Size:");
                    ui.horizontal(|ui| {
                        for (v, name) in [
                            (ShapeSize::Small, "Small"),
                            (ShapeSize::Medium, "Medium"),
                            (ShapeSize::Large, "Large"),
                        ] {
                            if ui.radio(m.size == v, name).clicked() && m.size != v {
                                set(actions, json!({ "size": v }));
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("Colour:");
                    ui.horizontal(|ui| {
                        colour_button(ui, m.shape_color, false, actions);
                        ui.label("Shape");
                        command_button_small(ui, engine, "motion.swap_colors", actions);
                        colour_button(ui, m.background, true, actions);
                        ui.label("Background");
                    });
                    ui.end_row();

                    ui.label("Move:");
                    ui.horizontal(|ui| {
                        for (v, name) in [
                            (MoveStep::PixelByPixel, "Pixel by pixel"),
                            (MoveStep::ShapeByShape, "Shape by shape"),
                        ] {
                            if ui.radio(m.step == v, name).clicked() && m.step != v {
                                set(actions, json!({ "step": v }));
                            }
                        }
                    });
                    ui.end_row();
                });
        });
    });
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        command_button(ui, engine, "view.close", actions);
    });
}

/// A colour swatch that opens a picker; `background` picks which colour it sets.
fn colour_button(ui: &mut egui::Ui, c: Rgb8, background: bool, actions: &mut Vec<Action>) {
    let mut rgb = [c.r, c.g, c.b];
    if egui::color_picker::color_edit_button_srgb(ui, &mut rgb).changed() {
        let [r, g, b] = rgb;
        let c = json!({ "r": r, "g": g, "b": b });
        set(
            actions,
            if background {
                json!({ "background": c })
            } else {
                json!({ "shape_color": c })
            },
        );
    }
}

/// Draw the shape at its position for `elapsed` seconds, on whole physical pixels.
fn paint_shape(ui: &egui::Ui, m: &Motion, rect: egui::Rect, elapsed: f64) {
    let ppp = ui.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let size = snap(m.size.points()).min(rect.width()).min(rect.height());
    let step = snap(m.step_points()).max(1.0 / ppp);
    let updates = (elapsed * f64::from(m.speed.rate_hz())).floor();
    let updates = if updates.is_finite() && updates >= 0.0 {
        updates as u64
    } else {
        0
    };
    let x = motion_x(updates, step, rect.width() - size);
    let min = egui::pos2(snap(rect.left() + x), snap(rect.center().y - size / 2.0));
    let shape = egui::Rect::from_min_size(min, egui::vec2(size, size));
    let fill = color(m.shape_color);
    let painter = ui.painter();
    let line = snap((size / 10.0).max(1.0));
    match (m.shape, m.hollow) {
        (Shape::Square, false) => {
            painter.rect_filled(shape, 0.0, fill);
        }
        (Shape::Square, true) => {
            let stroke = egui::Stroke::new(line, fill);
            painter.rect_stroke(shape, 0.0, stroke, egui::StrokeKind::Inside);
        }
        (Shape::Circle, false) => {
            painter.circle_filled(shape.center(), size / 2.0, fill);
        }
        (Shape::Circle, true) => {
            let stroke = egui::Stroke::new(line, fill);
            painter.circle_stroke(shape.center(), (size - line) / 2.0, stroke);
        }
    }
}
