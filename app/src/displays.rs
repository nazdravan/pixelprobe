//! Read the displays from the platform: winit everywhere, plus names, physical size and the
//! current mode from AppKit / CoreGraphics on macOS. No `unsafe`.

use pp_ui::pp_engine::pp_core::{DisplayInfo, DisplayMode};
use winit::window::Window;

/// Every display, the primary first. `window` tells which display shows the app.
pub fn list(window: &Window) -> Vec<DisplayInfo> {
    let primary = window.primary_monitor();
    let current = window.current_monitor();
    let mut out: Vec<DisplayInfo> = window
        .available_monitors()
        .take(pp_ui::pp_engine::pp_core::display::MAX_DISPLAYS)
        .map(|m| {
            let size = m.size();
            let pos = m.position();
            let modes = m
                .video_modes()
                .take(pp_ui::pp_engine::pp_core::display::MAX_MODES * 2)
                .map(|v| DisplayMode {
                    width: v.size().width,
                    height: v.size().height,
                    bits: v.bit_depth(),
                    refresh_mhz: v.refresh_rate_millihertz(),
                })
                .collect::<Vec<_>>();
            let refresh_mhz = m.refresh_rate_millihertz().unwrap_or(0);
            // The current mode's bit depth: a listed mode with the same size and rate.
            let bits = modes
                .iter()
                .find(|v| {
                    v.width == size.width && v.height == size.height && v.refresh_mhz == refresh_mhz
                })
                .or_else(|| {
                    modes
                        .iter()
                        .find(|v| v.width == size.width && v.height == size.height)
                })
                .map_or(0, |v| v.bits);
            #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
            let mut info = DisplayInfo {
                name: m.name().unwrap_or_default(),
                primary: primary.as_ref() == Some(&m),
                has_window: current.as_ref() == Some(&m),
                x: pos.x,
                y: pos.y,
                width: size.width,
                height: size.height,
                scale: m.scale_factor(),
                refresh_mhz,
                bits,
                modes,
                ..Default::default()
            };
            #[cfg(target_os = "macos")]
            macos::fill(&m, &mut info);
            info
        })
        .collect();
    // Primary first, then left to right, top to bottom.
    out.sort_by_key(|d| (!d.primary, d.x, d.y));
    out
}

#[cfg(target_os = "macos")]
mod macos {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;
    use objc2_core_graphics::{
        CGDisplayCopyDisplayMode, CGDisplayIsBuiltin, CGDisplayIsMain, CGDisplayMode,
        CGDisplayRotation, CGDisplayScreenSize,
    };
    use objc2_foundation::{NSNumber, NSString};
    use pp_ui::pp_engine::pp_core::DisplayInfo;
    use winit::monitor::MonitorHandle;
    use winit::platform::macos::MonitorHandleExtMacOS;

    /// Add what winit does not report on macOS.
    pub fn fill(m: &MonitorHandle, info: &mut DisplayInfo) {
        let id = m.native_id();
        let mm = CGDisplayScreenSize(id);
        info.width_mm = mm.width;
        info.height_mm = mm.height;
        info.primary = CGDisplayIsMain(id);
        info.builtin = CGDisplayIsBuiltin(id);
        info.rotation = CGDisplayRotation(id);
        if let Some(mode) = CGDisplayCopyDisplayMode(id) {
            let rate = CGDisplayMode::refresh_rate(Some(&mode));
            // Built-in panels report 0 Hz here; keep winit's value then.
            if rate.is_finite() && rate > 0.0 {
                info.refresh_mhz = (rate * 1000.0).round().min(f64::from(u32::MAX)) as u32;
            }
        }
        if let Some(name) = screen_name(id) {
            info.name = name;
        }
    }

    /// The display's name as macOS shows it ("Built-in Retina Display", "DELL U2720Q").
    fn screen_name(id: u32) -> Option<String> {
        // AppKit is main-thread only; eframe draws on the main thread.
        let mtm = MainThreadMarker::new()?;
        let key = NSString::from_str("NSScreenNumber");
        NSScreen::screens(mtm).iter().find_map(|screen| {
            let number = screen.deviceDescription().objectForKey(&key)?;
            let number = number.downcast::<NSNumber>().ok()?;
            (number.unsignedIntValue() == id).then(|| screen.localizedName().to_string())
        })
    }
}
