# Changelog

All notable changes to pixelprobe. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the version numbers
[Semantic Versioning](https://semver.org/). Each version's section is also the text of its
GitHub release.

## [0.1.0] — 2026-11-01

### Added

- Display information: every monitor with its name, graphics adapter, resolution, scale,
  refresh rate, colour depth, physical size, diagonal, pixels per inch, aspect ratios,
  position and rotation, and the display modes it supports. On Windows and Linux the size,
  names and adapter come from the monitors' EDID.
- Pattern test: 15 full-screen patterns at the display's own resolution, with SMPTE RP 219
  colour bars and their sub-pattern options.
- Motion test: a square or circle, filled or hollow, three sizes, shape and background
  colours, pixel or shape steps, 5 to 240 steps per second, with the measured drawing rate.
- Pixel refresh: random red, green, blue, white and black cells, 1 to 60 updates per second,
  in a window or full screen.
- Tests full screen on any monitor, optionally in a chosen display mode.
- Dark and light theme following the system, keyboard shortcuts, undo for every setting,
  help screen (F1).
- Web version (`cargo xtask web`).
- Packages: Linux AppImage, `.deb` and `.tar.gz`; Windows `.msi` and portable `.zip`;
  macOS universal `.dmg`.

[0.1.0]: https://github.com/nazdravan/pixelprobe/releases/tag/v0.1.0
