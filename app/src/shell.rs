//! One frame of the app, shared by the desktop and web builds: the status line, the panels
//! behind a panic guard, and running what the user triggered.

use pp_ui::egui;
use pp_ui::pp_engine::Engine;
use pp_ui::pp_engine::pp_core::Screen;

pub fn frame(ui: &mut egui::Ui, engine: &mut Engine, status: &mut Option<String>) {
    // Only the motion test keeps its controls in full screen.
    let doc = engine.doc();
    let hide_status = doc.wants_full_screen() && doc.screen != Screen::MotionTest;
    if let Some(msg) = status.as_ref().filter(|_| !hide_status) {
        let mut dismiss = false;
        egui::Panel::top("status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(ui.visuals().warn_fg_color, msg);
                dismiss = ui.small_button("×").clicked();
            });
        });
        if dismiss {
            *status = None;
        }
    }
    // Panic guard for the frame: panels only read the engine, so a panic while drawing
    // cannot corrupt the document; report it and keep running. (On wasm panics abort.)
    let shown = &*engine;
    let drawn = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pp_ui::show(ui, shown)));
    match drawn {
        Ok(actions) => {
            for action in actions {
                if let Err(e) = engine.run(action.id, action.params) {
                    *status = Some(e.to_string());
                }
            }
        }
        Err(_) => *status = Some("internal error while drawing; see the log".into()),
    }
}
