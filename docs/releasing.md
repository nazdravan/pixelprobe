# Making a release

Releases are made **automatically** by GitHub Actions (`.github/workflows/release.yml`) on
every push to `main`:

- the tests run;
- if the version in `Cargo.toml` is **not yet published**, packages are built for every
  platform and the release "pixelprobe v&lt;version&gt;" is published, with the tag
  `v<version>`, `SHA256SUMS.txt` and the version's notes from `CHANGELOG.md` followed by
  GitHub's list of changes. The release is created as a draft, every file is uploaded, and
  only then is it made public;
- if the version is already published, nothing else is built: a normal push does not make a
  release, and a published release is never overwritten;
- if a build or the tests fail, nothing is published;
- a version with a suffix (`0.2.0-rc.1`) is published as a *pre-release*.

## The version

The version lives in one place: `version` in the `[workspace.package]` section of
`Cargo.toml`.

```sh
packaging/version.sh              # prints the version
packaging/version.sh set 0.2.0    # changes it in Cargo.toml and Cargo.lock
```

## The changelog

Every version has a section `## [0.2.0] — <date>` in `CHANGELOG.md`, with the headings
*Added*, *Changed*, *Fixed*. That section becomes the release text; without it the workflow
stops before the builds.

```sh
packaging/changelog.sh            # the notes of the version in Cargo.toml
packaging/changelog.sh 0.2.0      # another version's notes
```

## Steps

1. On `main`, raise the version, write its section in `CHANGELOG.md`, commit and push:
   ```sh
   packaging/version.sh set 0.2.0
   git commit -am "Version 0.2.0"
   git push
   ```
2. Follow the **release** workflow in the *Actions* tab. When it finishes, the release is in
   *Releases*.

If a build fails, fix it and push to `main` again with the same version: as long as the
version is unpublished, every push retries the release. If a published release has a mistake,
make a new version (`0.2.1`).

The commit a release is made from must not change files in `.github/workflows/`: the workflow
token may not create a tag on such a commit (`HTTP 403: Resource not accessible by
integration`). The workflow checks this: on such a commit it builds the packages but leaves the
release to the next commit that does not touch the workflows. Push workflow changes in their
own commit, before the version commit.

## Test run

*Actions → release → Run workflow* runs the workflow on any branch, optionally with a test
version (e.g. `0.2.0-rc.1`). Only the tests and the builds run; the packages stay as artifacts
of the run and no release is created.

## What gets built

| File | For | Built by |
|---|---|---|
| `pixelprobe-<v>-linux-x86_64.AppImage` | Linux x86_64, one executable file | `packaging/linux/appimage.sh`, ubuntu-22.04 |
| `pixelprobe-<v>-linux-x86_64.deb` | Ubuntu / Debian / Mint (Ubuntu 22.04 and newer), with a menu entry | `packaging/linux/deb.sh`, ubuntu-22.04 |
| `pixelprobe-<v>-linux-x86_64.tar.gz` | Linux x86_64, plain archive | `packaging/linux/package.sh`, ubuntu-22.04 |
| `pixelprobe-<v>-macos-universal.dmg` | macOS 10.13 and newer, Apple Silicon and Intel (`pixelprobe.app`) | `packaging/macos/package.sh`, macos-26 |
| `pixelprobe-<v>-windows-x64.msi` | Windows 10/11 x64 installer: Program Files, Start menu, upgrades the installed version | `packaging/windows/msi.ps1` (WiX Toolset 5.0.2) |
| `pixelprobe-<v>-windows-x64-portable.zip` | Windows 10/11 x64, no installation | `packaging/windows/package.ps1` |
| `SHA256SUMS.txt` | checksums of the files above | the `release` job |

Every script in `packaging/` also runs locally (the macOS one only on a Mac); the output goes
to `dist/release/`.

## Good to know

- **The files are not signed.** On first start (and when installing the `.msi`), Windows
  SmartScreen shows "Windows protected your PC": *More info → Run anyway*. On macOS the app
  has only an ad-hoc signature, so Gatekeeper blocks the first start; the README's macOS
  section says how to open it. Real signing and notarization need paid certificates.
- The `.msi` gets only the numeric part of the version (`0.2.0-rc.1` → `0.2.0`), since MSI
  accepts nothing else. The `UpgradeCode` in `packaging/windows/pixelprobe.wxs` never changes:
  by it, a new `.msi` replaces the installed version.
- `appimagetool` is downloaded into `.cache/` at a fixed release and checked against its
  SHA-256 (`packaging/linux/appimage.sh`). The AppImage holds only the program; the graphics
  libraries (OpenGL, X11/Wayland) come from the system. Built on Ubuntu 22.04, it runs on
  distributions with glibc 2.35 or newer.
- The GitHub actions are pinned by commit SHA. Update them by hand, together with the version
  comment next to the SHA.
