use pp_control::ControlServer;
use pp_ui::egui;
use pp_ui::pp_engine::Engine;
use pp_ui::pp_engine::pp_core::Screen;
use std::process::ExitCode;

/// Parsed command line.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Args {
    pub control: Option<u16>,
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut out = Args::default();
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--control" => {
                let port = it.next().ok_or("--control needs a PORT")?;
                let port = port
                    .parse::<u16>()
                    .map_err(|_| format!("--control: `{port}` is not a port number"))?;
                out.control = Some(port);
            }
            "-h" | "--help" => return Err("usage: pixelprobe [--control PORT]".into()),
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(out)
}

pub fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("pixelprobe")
            .with_inner_size([740.0, 620.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "pixelprobe",
        options,
        Box::new(move |cc| {
            let control = match args.control {
                None => None,
                Some(port) => {
                    let ctx = cc.egui_ctx.clone();
                    let server = ControlServer::start(port, move || ctx.request_repaint())?;
                    eprintln!("pixelprobe: control channel on {}", server.addr());
                    Some(server)
                }
            };
            Ok(Box::new(App {
                engine: Engine::new(),
                control,
                status: None,
                last_screen: Screen::Main,
                displays_read: None,
            }))
        }),
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("pixelprobe: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Screens shown full screen.
fn full_screen(screen: Screen) -> bool {
    matches!(screen, Screen::PatternTest | Screen::MotionTest)
}

struct App {
    engine: Engine,
    control: Option<ControlServer>,
    /// Last command error or caught UI panic, shown in a status line.
    status: Option<String>,
    /// Screen shown last frame, to react to screen changes (full screen on/off).
    last_screen: Screen,
    /// When the displays were last read and the `display.refresh` count seen then.
    displays_read: Option<(f64, u64)>,
}

/// Re-read the displays this often, in seconds, to follow plugging and window moves.
const DISPLAY_POLL_SECS: f64 = 3.0;

impl App {
    /// The pattern and motion tests run full screen on the selected monitor (in the selected
    /// mode with "Set test mode as selection"); leaving them restores the window.
    fn sync_viewport(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let screen = self.engine.doc().screen;
        if screen == self.last_screen {
            return;
        }
        let (now, before) = (full_screen(screen), full_screen(self.last_screen));
        if now != before {
            match frame.winit_window() {
                Some(window) => {
                    let target = now.then(|| {
                        let doc = self.engine.doc();
                        let info = self.engine.selected_display();
                        let mode = doc
                            .test_in_mode
                            .then(|| info.and_then(|d| doc.mode.and_then(|m| d.modes.get(m))))
                            .flatten();
                        crate::displays::fullscreen(window, info, mode)
                    });
                    window.set_fullscreen(target);
                }
                // Headless or unknown backend: let eframe do plain full screen.
                None => ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(now)),
            }
        }
        self.last_screen = screen;
    }
}

impl App {
    /// Read the displays at start, on `display.refresh`, and every few seconds.
    fn read_displays(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let Some(window) = frame.winit_window() else {
            return;
        };
        let now = ctx.input(|i| i.time);
        let refreshes = self.engine.session().display_refreshes;
        let due = match self.displays_read {
            None => true,
            Some((at, seen)) => seen != refreshes || now - at >= DISPLAY_POLL_SECS,
        };
        if due {
            let displays = crate::displays::list(window);
            if displays != self.engine.displays() {
                self.engine.set_displays(displays);
            }
            self.displays_read = Some((now, refreshes));
        }
        ctx.request_repaint_after_secs(DISPLAY_POLL_SECS as f32);
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(server) = &self.control {
            for pending in server.drain() {
                pp_control::handle(&mut self.engine, ctx, pending);
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.read_displays(ui.ctx(), frame);
        self.sync_viewport(ui.ctx(), frame);
        let hide_status = self.engine.doc().screen == Screen::PatternTest;
        if let Some(msg) = self.status.as_ref().filter(|_| !hide_status) {
            let mut dismiss = false;
            egui::Panel::top("status").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(ui.visuals().warn_fg_color, msg);
                    dismiss = ui.small_button("×").clicked();
                });
            });
            if dismiss {
                self.status = None;
            }
        }
        // Panic guard for the frame: panels only read the engine, so a panic while drawing
        // cannot corrupt the document; report it and keep running.
        let engine = &self.engine;
        let drawn =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pp_ui::show(ui, engine)));
        match drawn {
            Ok(actions) => {
                for action in actions {
                    if let Err(e) = self.engine.run(action.id, action.params) {
                        self.status = Some(e.to_string());
                    }
                }
            }
            Err(_) => self.status = Some("internal error while drawing; see the log".into()),
        }
        self.sync_viewport(ui.ctx(), frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(args: &[&str]) -> Result<Args, String> {
        parse_args(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn args() {
        assert_eq!(p(&[]), Ok(Args::default()));
        assert_eq!(
            p(&["--control", "7878"]),
            Ok(Args {
                control: Some(7878)
            })
        );
        assert!(p(&["--control"]).is_err());
        assert!(p(&["--control", "99999"]).is_err());
        assert!(p(&["--bogus"]).is_err());
    }
}
