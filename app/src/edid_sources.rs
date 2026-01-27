//! Where the OS keeps the monitors' EDIDs and the adapters' names, outside the windowing
//! system: sysfs on Linux, the registry on Windows. Everything read is bounded; anything
//! unreadable is skipped. Parsing and matching live in `pp-edid`.

use pp_edid::Source;

/// Most entries looked at in any one directory or registry key.
#[cfg(any(target_os = "linux", windows))]
const MAX_ENTRIES: usize = 256;

/// (one source per connected monitor, every adapter name).
pub fn sources() -> (Vec<Source>, Vec<String>) {
    platform::sources()
}

#[cfg(target_os = "linux")]
mod platform {
    use super::MAX_ENTRIES;
    use pp_edid::{MAX_LEN, Source, fill::pci_vendor};
    use std::io::Read;
    use std::path::Path;

    const DRM: &str = "/sys/class/drm";

    fn read_capped(path: &Path) -> Vec<u8> {
        let mut out = Vec::new();
        if let Ok(f) = std::fs::File::open(path) {
            let _ = f.take(MAX_LEN as u64).read_to_end(&mut out);
        }
        out
    }

    fn read_text(path: &Path) -> String {
        String::from_utf8_lossy(&read_capped(path))
            .trim()
            .to_owned()
    }

    /// "AMD (amdgpu)" from `cardN/device/vendor` and the driver in `cardN/device/uevent`.
    fn gpu_name(card: &Path) -> Option<String> {
        let device = card.join("device");
        let vendor = read_text(&device.join("vendor"));
        let vendor = u16::from_str_radix(vendor.trim_start_matches("0x"), 16).ok()?;
        let vendor = pci_vendor(vendor).map_or(format!("PCI vendor {vendor:04x}"), str::to_owned);
        let uevent = read_text(&device.join("uevent"));
        let driver = uevent.lines().find_map(|l| l.strip_prefix("DRIVER="));
        Some(match driver {
            Some(d) => format!("{vendor} ({d})"),
            None => vendor,
        })
    }

    pub fn sources() -> (Vec<Source>, Vec<String>) {
        sources_in(Path::new(DRM))
    }

    /// Read the DRM class directory at `drm` (normally `/sys/class/drm`).
    pub fn sources_in(drm: &Path) -> (Vec<Source>, Vec<String>) {
        let (mut sources, mut gpus) = (Vec::new(), Vec::new());
        let Ok(dir) = std::fs::read_dir(drm) else {
            return (sources, gpus);
        };
        for entry in dir.flatten().take(MAX_ENTRIES) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            // Connectors are `card0-HDMI-A-1`; adapters are `card0`.
            if let Some((card, _)) = name.split_once('-') {
                if read_text(&path.join("status")) != "connected" {
                    continue;
                }
                sources.push(Source {
                    edid: read_capped(&path.join("edid")),
                    gpu: gpu_name(&drm.join(card)),
                });
            } else if name.starts_with("card")
                && let Some(gpu) = gpu_name(&path)
                && !gpus.contains(&gpu)
            {
                gpus.push(gpu);
            }
        }
        (sources, gpus)
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::platform::sources_in;
    use std::fs;

    #[test]
    fn reads_a_sysfs_tree() {
        let root = std::env::temp_dir().join(format!("pp-drm-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let card = root.join("card0/device");
        fs::create_dir_all(&card).unwrap();
        fs::write(card.join("vendor"), "0x1002\n").unwrap();
        fs::write(card.join("uevent"), "DRIVER=amdgpu\nPCI_ID=1002:67DF\n").unwrap();
        let hdmi = root.join("card0-HDMI-A-1");
        fs::create_dir_all(&hdmi).unwrap();
        fs::write(hdmi.join("status"), "connected\n").unwrap();
        let edid = include_bytes!("../../crates/pp-edid/tests/data/imac-21-4k.bin");
        fs::write(hdmi.join("edid"), edid).unwrap();
        let dp = root.join("card0-DP-1");
        fs::create_dir_all(&dp).unwrap();
        fs::write(dp.join("status"), "disconnected\n").unwrap();

        let (sources, gpus) = sources_in(&root);
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(gpus, ["AMD (amdgpu)"]);
        assert_eq!(sources.len(), 1, "only the connected connector");
        assert_eq!(sources[0].gpu.as_deref(), Some("AMD (amdgpu)"));
        assert_eq!(sources[0].edid, edid);
        let parsed = pp_edid::parse(&sources[0].edid).unwrap();
        assert_eq!((parsed.width_mm, parsed.height_mm), (480, 270));
    }
}

#[cfg(windows)]
mod platform {
    use super::MAX_ENTRIES;
    use pp_edid::{MAX_LEN, Source};
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    /// Monitors seen by Windows; an instance with a `Control` subkey is connected now.
    const MONITORS: &str = r"SYSTEM\CurrentControlSet\Enum\DISPLAY";
    /// The display adapter device class.
    const ADAPTERS: &str =
        r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";

    pub fn sources() -> (Vec<Source>, Vec<String>) {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let mut sources = Vec::new();
        if let Ok(display) = hklm.open_subkey(MONITORS) {
            for model in display.enum_keys().flatten().take(MAX_ENTRIES) {
                let Ok(model) = display.open_subkey(&model) else {
                    continue;
                };
                for instance in model.enum_keys().flatten().take(MAX_ENTRIES) {
                    let Ok(instance) = model.open_subkey(&instance) else {
                        continue;
                    };
                    if instance.open_subkey("Control").is_err() {
                        continue;
                    }
                    let Ok(value) = instance
                        .open_subkey("Device Parameters")
                        .and_then(|k| k.get_raw_value("EDID"))
                    else {
                        continue;
                    };
                    let mut edid = value.bytes.to_vec();
                    edid.truncate(MAX_LEN);
                    sources.push(Source { edid, gpu: None });
                }
            }
        }
        let mut gpus = Vec::new();
        if let Ok(class) = hklm.open_subkey(ADAPTERS) {
            for key in class.enum_keys().flatten().take(MAX_ENTRIES) {
                let Ok(adapter) = class.open_subkey(&key) else {
                    continue;
                };
                if let Ok(desc) = adapter.get_value::<String, _>("DriverDesc")
                    && !gpus.contains(&desc)
                {
                    gpus.push(desc);
                }
            }
        }
        (sources, gpus)
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod platform {
    use pp_edid::Source;

    /// macOS reads size, names and GPUs from CoreGraphics instead.
    pub fn sources() -> (Vec<Source>, Vec<String>) {
        (Vec::new(), Vec::new())
    }
}
