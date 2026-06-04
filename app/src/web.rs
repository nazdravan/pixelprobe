//! Web build: eframe on the page's `pixelprobe` canvas. The display comes from
//! `window.screen`; full screen uses the browser Fullscreen API.

use eframe::wasm_bindgen::JsCast;
use eframe::web_sys;
use pp_ui::egui;
use pp_ui::pp_engine::Engine;
use pp_ui::pp_engine::pp_core::DisplayInfo;

/// Id of the canvas in `index.html`.
const CANVAS_ID: &str = "pixelprobe";

pub fn main() {
    wasm_bindgen_futures::spawn_local(async {
        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let Some(canvas) = document
            .get_element_by_id(CANVAS_ID)
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        else {
            web_sys::console::error_1(&format!("pixelprobe: no <canvas id={CANVAS_ID}>").into());
            return;
        };
        let started = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    pp_ui::theme::apply(&cc.egui_ctx);
                    Ok(Box::new(WebApp::default()))
                }),
            )
            .await;
        if let Err(e) = started {
            web_sys::console::error_2(&"pixelprobe: failed to start".into(), &e);
        }
    });
}

#[derive(Default)]
struct WebApp {
    engine: Engine,
    status: Option<String>,
    full_screen: bool,
    /// The `display.refresh` count when the screen was last read.
    displays_read: Option<u64>,
}

/// The screen the browser runs on. Browsers report CSS pixels; times the device pixel
/// ratio gives physical pixels. Refresh rate and physical size are not exposed.
fn screen_info() -> Option<DisplayInfo> {
    let window = web_sys::window()?;
    let screen = window.screen().ok()?;
    let ratio = window.device_pixel_ratio();
    let px = |css: i32| {
        let v = (f64::from(css) * ratio).round();
        if v.is_finite() && v > 0.0 {
            v.min(f64::from(u32::MAX)) as u32
        } else {
            0
        }
    };
    Some(DisplayInfo {
        name: "Browser screen".into(),
        primary: true,
        has_window: true,
        width: px(screen.width().ok()?),
        height: px(screen.height().ok()?),
        scale: ratio,
        bits: screen
            .color_depth()
            .ok()
            .and_then(|b| u16::try_from(b).ok())
            .unwrap_or(0),
        ..Default::default()
    })
}

/// Enter or leave browser full screen. Browsers allow entering only shortly after a click
/// or key press, which is when the test commands run; a refusal is reported in the log.
fn set_full_screen(on: bool) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let result = if on {
        document
            .document_element()
            .map_or(Ok(()), |e| e.request_fullscreen())
    } else if document.fullscreen_element().is_some() {
        document.exit_fullscreen();
        Ok(())
    } else {
        Ok(())
    };
    if let Err(e) = result {
        web_sys::console::warn_2(&"pixelprobe: full screen refused".into(), &e);
    }
}

impl eframe::App for WebApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let refreshes = self.engine.session().display_refreshes;
        if self.displays_read != Some(refreshes) {
            self.engine
                .set_displays(screen_info().into_iter().collect());
            self.displays_read = Some(refreshes);
        }
        crate::shell::frame(ui, &mut self.engine, &mut self.status);
        let want = self.engine.doc().wants_full_screen();
        if want != self.full_screen {
            set_full_screen(want);
            self.full_screen = want;
        }
    }
}
