use pp_control::ControlServer;
use pp_ui::egui;
use pp_ui::pp_engine::Engine;
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

struct App {
    engine: Engine,
    control: Option<ControlServer>,
    /// Last command error or caught UI panic, shown in a status line.
    status: Option<String>,
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(server) = &self.control {
            for pending in server.drain() {
                pp_control::handle(&mut self.engine, ctx, pending);
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(msg) = &self.status {
            let mut dismiss = false;
            egui::Panel::top("status").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(ui.visuals().warn_fg_color, msg);
                    dismiss = ui.small_button("✕").clicked();
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
