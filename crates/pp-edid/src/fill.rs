//! Fill in what the windowing system does not report (physical size, a real name, the GPU)
//! from the EDIDs the OS exposes, matching each EDID to a display.

use crate::{Edid, parse};
use pp_core::DisplayInfo;

/// One monitor as the OS exposes it outside the windowing system.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Source {
    /// Raw EDID bytes (may be empty or garbage: they are parsed and checked here).
    pub edid: Vec<u8>,
    /// The graphics adapter the monitor is connected to, when known.
    pub gpu: Option<String>,
}

/// Names that windowing systems give when they know no better (`\\.\DISPLAY1`, `Monitor #…`).
fn generic_name(name: &str) -> bool {
    name.is_empty() || name.starts_with(r"\\.\") || name.starts_with("Monitor #")
}

/// The largest mode of a display: its native resolution.
fn native(d: &DisplayInfo) -> (u32, u32) {
    d.modes
        .iter()
        .map(|m| (m.width, m.height))
        .chain(std::iter::once((d.width, d.height)))
        .max_by_key(|(w, h)| u64::from(*w) * u64::from(*h))
        .unwrap_or((d.width, d.height))
}

/// Match `sources` to `displays` and fill unknown fields. A source matches a display when
/// its EDID's preferred resolution is the display's native one and no other display has the
/// same native resolution; with one display and one source they match directly. Known values
/// are never overwritten. `all_gpus` is used when there is exactly one adapter.
pub fn fill(displays: &mut [DisplayInfo], sources: &[Source], all_gpus: &[String]) {
    let parsed: Vec<(Edid, Option<&String>)> = sources
        .iter()
        .filter_map(|s| parse(&s.edid).ok().map(|e| (e, s.gpu.as_ref())))
        .collect();
    let natives: Vec<(u32, u32)> = displays.iter().map(native).collect();
    let single = displays.len() == 1 && parsed.len() == 1;
    for (i, d) in displays.iter_mut().enumerate() {
        let res = natives.get(i).copied().unwrap_or_default();
        let unique_res = natives.iter().filter(|r| **r == res).count() == 1;
        let matched = if single {
            parsed.first()
        } else if unique_res {
            let mut hits = parsed.iter().filter(|(e, _)| e.preferred == Some(res));
            match (hits.next(), hits.next()) {
                (Some(hit), None) => Some(hit),
                _ => None,
            }
        } else {
            None
        };
        if let Some((edid, gpu)) = matched {
            if d.width_mm <= 0.0 || d.height_mm <= 0.0 {
                d.width_mm = f64::from(edid.width_mm);
                d.height_mm = f64::from(edid.height_mm);
            }
            if generic_name(&d.name)
                && let Some(name) = &edid.name
            {
                d.name = name.clone();
            }
            if d.gpu.is_empty()
                && let Some(gpu) = gpu
            {
                d.gpu = (*gpu).clone();
            }
        }
        if d.gpu.is_empty()
            && let [only] = all_gpus
        {
            d.gpu = only.clone();
        }
    }
}

/// Human name for a PCI vendor id (as in `/sys/class/drm/card0/device/vendor`).
pub fn pci_vendor(id: u16) -> Option<&'static str> {
    match id {
        0x1002 => Some("AMD"),
        0x10DE => Some("NVIDIA"),
        0x8086 => Some("Intel"),
        0x106B => Some("Apple"),
        0x15AD => Some("VMware"),
        0x1AF4 => Some("Red Hat (virtio)"),
        0x1234 => Some("QEMU"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pp_core::DisplayMode;

    const IMAC: &[u8] = include_bytes!("../tests/data/imac-21-4k.bin");

    fn display(name: &str, w: u32, h: u32) -> DisplayInfo {
        DisplayInfo {
            name: name.into(),
            width: w,
            height: h,
            modes: vec![DisplayMode {
                width: w,
                height: h,
                bits: 32,
                refresh_mhz: 60_000,
            }],
            ..Default::default()
        }
    }

    fn source(edid: Vec<u8>, gpu: &str) -> Source {
        Source {
            edid,
            gpu: Some(gpu.into()),
        }
    }

    #[test]
    fn single_display_takes_the_only_edid() {
        let mut ds = vec![display(r"\\.\DISPLAY1", 4096, 2304)];
        fill(&mut ds, &[source(IMAC.to_vec(), "AMD (amdgpu)")], &[]);
        assert_eq!((ds[0].width_mm, ds[0].height_mm), (480.0, 270.0));
        assert_eq!(ds[0].name, "iMac");
        assert_eq!(ds[0].gpu, "AMD (amdgpu)");
    }

    #[test]
    fn several_displays_match_by_preferred_resolution() {
        let wide = crate::tests::synthetic(); // preferred 2560 × 1440, 597 × 336 mm
        let mut ds = vec![display("HDMI-1", 1920, 1080), display("DP-1", 2560, 1440)];
        fill(
            &mut ds,
            &[source(IMAC.to_vec(), "x"), source(wide, "NVIDIA (nvidia)")],
            &["a".into(), "b".into()],
        );
        assert_eq!((ds[1].width_mm, ds[1].height_mm), (597.0, 336.0));
        assert_eq!(ds[1].name, "DP-1", "a real name is kept");
        assert_eq!(ds[1].gpu, "NVIDIA (nvidia)");
        // No EDID with a 1920 × 1080 preferred mode: left unknown, and two GPUs: no guess.
        assert_eq!((ds[0].width_mm, ds[0].gpu.as_str()), (0.0, ""));
    }

    #[test]
    fn known_values_and_ambiguity() {
        let mut ds = vec![DisplayInfo {
            width_mm: 100.0,
            height_mm: 50.0,
            ..display("Mine", 4096, 2304)
        }];
        fill(&mut ds, &[source(IMAC.to_vec(), "g")], &[]);
        assert_eq!((ds[0].width_mm, ds[0].name.as_str()), (100.0, "Mine"));
        // Two displays with the same native resolution: never guess.
        let wide = crate::tests::synthetic();
        let mut ds = vec![display("A", 2560, 1440), display("B", 2560, 1440)];
        fill(&mut ds, &[source(wide, "g")], &["Only GPU".into()]);
        assert!(ds.iter().all(|d| d.width_mm == 0.0));
        assert!(
            ds.iter().all(|d| d.gpu == "Only GPU"),
            "one adapter: it drives all"
        );
        // Garbage EDIDs are ignored.
        let mut ds = vec![display("A", 800, 600)];
        fill(&mut ds, &[source(vec![1, 2, 3], "g")], &[]);
        assert_eq!(ds[0].width_mm, 0.0);
    }

    #[test]
    fn vendors() {
        assert_eq!(pci_vendor(0x10de), Some("NVIDIA"));
        assert_eq!(pci_vendor(0xffff), None);
    }
}
