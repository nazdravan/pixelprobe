//! pixelprobe desktop binary.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: pp_ui::egui::ViewportBuilder::default()
            .with_title("pixelprobe")
            .with_inner_size([740.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native("pixelprobe", options, Box::new(|_cc| Ok(Box::new(App))))
}

// The web entry point is a separate gap; the wasm target only has to compile.
#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
struct App;

#[cfg(not(target_arch = "wasm32"))]
impl eframe::App for App {
    fn ui(&mut self, ui: &mut pp_ui::egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("pixelprobe");
    }
}
