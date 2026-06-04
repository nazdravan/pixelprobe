#!/usr/bin/env python3
"""Builds the landing page (GitHub Pages) from site/, assets/theme.toml, CHANGELOG.md, the
screenshots in docs/assets and, when there is one, the latest release. Standard library only.

  python3 site/build.py --out _site [--release release.json] [--web target/web]

release.json: `gh release view --json tagName,publishedAt,url,assets`. Without it the page
says no release is out yet. With it, a missing package fails the build, so the page already
online (with working links) stays. --web: the web build (`cargo xtask web`), served at app/.
"""
import argparse
import html
import json
import re
import shutil
import tomllib
from pathlib import Path

SITE = Path(__file__).resolve().parent
ROOT = SITE.parent
REPO = "https://github.com/nazdravan/pixelprobe"
PAGE = "https://nazdravan.github.io/pixelprobe/"

# The release files the page links to: key -> end of the name after `pixelprobe-<version>-`
# (SHA256SUMS.txt has the whole name).
ASSET_KINDS = {
    "win-msi": "windows-x64.msi",
    "win-zip": "windows-x64-portable.zip",
    "mac-dmg": "macos-universal.dmg",
    "linux-appimage": "linux-x86_64.AppImage",
    "linux-deb": "linux-x86_64.deb",
    "linux-tar": "linux-x86_64.tar.gz",
    "sha256": "SHA256SUMS.txt",
}
# Shown in the download table, in this order.
DOWNLOADS = [
    ("win-msi", "Windows 10/11", "installer"),
    ("win-zip", "Windows 10/11", "portable, no installation"),
    ("mac-dmg", "macOS 10.13+", "Apple Silicon and Intel"),
    ("linux-appimage", "Linux x86_64", "AppImage, one file"),
    ("linux-deb", "Ubuntu / Debian / Mint", "package"),
    ("linux-tar", "Linux x86_64", "archive"),
]
# The hero button per system: release file, button text, note under it.
PRIMARY = {
    "windows": ("win-msi", "Download for Windows", "Windows 10/11 x64 installer · {size}"),
    "mac": ("mac-dmg", "Download for macOS", "Apple Silicon and Intel · {size}"),
    "linux": ("linux-appimage", "Download for Linux", "AppImage, x86_64 · {size}"),
}
THEME_KEYS = ["accent", "accent_press", "accent_light", "bg_dark", "panel_dark", "fg_dark",
              "bg_light", "panel_light", "fg_light", "bezel", "badge_top", "badge_bottom"]
SECTION = re.compile(r"^## \[(?P<version>[^\]]+)\] — (?P<date>\d{4}-\d{2}-\d{2})$", re.M)


def theme_css(theme):
    """CSS custom properties, one per theme colour (--accent, --bg-dark, ...)."""
    lines = [f"  --{k.replace('_', '-')}: {theme[k]};" for k in THEME_KEYS]
    bars = "".join(f"  --bar-{i}: {c};\n" for i, c in enumerate(theme["bars"]))
    return ":root {\n" + "\n".join(lines) + "\n" + bars + "}\n"


def changelog_sections(md, count):
    """The first `count` versions in the changelog: version, date, body (Markdown)."""
    matches = list(SECTION.finditer(md))[:count]
    out = []
    for i, m in enumerate(matches):
        end = matches[i + 1].start() if i + 1 < len(matches) else len(md)
        body = md[m.end():end]
        cut = re.search(r"^## |^\[[^\]]+\]: \S+$", body, re.M)
        if cut:
            body = body[:cut.start()]
        out.append({"version": m["version"], "date": m["date"], "body": body.strip()})
    return out


def inline(text):
    """`code`, **bold** and [text](url) on plain text, escaped."""
    out = []
    for part in re.split(r"(`[^`]*`)", text):
        if len(part) > 1 and part.startswith("`") and part.endswith("`"):
            out.append(f"<code>{html.escape(part[1:-1], quote=False)}</code>")
            continue
        part = html.escape(part, quote=False)
        part = re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", part)
        part = re.sub(r"\[([^\]]+)\]\((https?://[^)\s]+)\)", r'<a href="\2">\1</a>', part)
        out.append(part)
    return "".join(out)


def markdown(md):
    """The changelog's Markdown subset: ### headings, - lists (with continuation lines), text."""
    out, item, in_list = [], None, False

    def flush():
        nonlocal item
        if item is not None:
            out.append(f"<li>{inline(item)}</li>")
            item = None

    for line in md.splitlines():
        if line.startswith("- "):
            flush()
            if not in_list:
                out.append("<ul>")
                in_list = True
            item = line[2:].strip()
        elif item is not None and line.startswith("  ") and line.strip():
            item += " " + line.strip()
        else:
            flush()
            if in_list:
                out.append("</ul>")
                in_list = False
            if line.startswith("### "):
                out.append(f"<h4>{inline(line[4:].strip())}</h4>")
            elif line.strip():
                out.append(f"<p>{inline(line.strip())}</p>")
    flush()
    if in_list:
        out.append("</ul>")
    return "\n".join(out)


def release_assets(release):
    """Key -> {name, url, size}. A missing file stops the build."""
    version = release["tagName"].removeprefix("v")
    by_name = {a["name"]: a for a in release["assets"]}
    assets = {}
    for kind, suffix in ASSET_KINDS.items():
        name = suffix if kind == "sha256" else f"pixelprobe-{version}-{suffix}"
        if name not in by_name:
            raise SystemExit(f"release {release['tagName']} has no file {name}")
        a = by_name[name]
        assets[kind] = {"name": name, "url": a["url"], "size": a["size"]}
    return assets


def size(n):
    return f"{n / 1024 / 1024:.1f} MB"


def render(template, values):
    """{{{ key }}}: raw HTML; {{ key }}: escaped text. An unknown key is an error."""
    def raw(m):
        return values[m[1]]

    def text(m):
        return html.escape(values[m[1]])

    page = re.sub(r"\{\{\{ (\w+) \}\}\}", raw, template)
    return re.sub(r"\{\{ (\w+) \}\}", text, page)


def page_values(release, web, changelog_md):
    values = {"repo": REPO, "page": PAGE}
    if release:
        assets = release_assets(release)
        version = release["tagName"].removeprefix("v")
        kinds = {
            system: {
                "url": assets[k]["url"],
                "label": label,
                "note": note.format(size=size(assets[k]["size"])),
            }
            for system, (k, label, note) in PRIMARY.items()
        }
        rows = "\n".join(
            f'<tr><td>{html.escape(sys)}</td><td>{html.escape(what)}</td>'
            f'<td><a href="{html.escape(assets[k]["url"])}">{html.escape(assets[k]["name"])}</a></td>'
            f'<td class="num">{size(assets[k]["size"])}</td></tr>'
            for k, sys, what in DOWNLOADS
        )
        values.update(
            version_line=f"Version {version} · {release['publishedAt'][:10]}",
            main_href="#download",
            main_label="Download",
            main_note="Windows, macOS and Linux",
            kinds_json=json.dumps(kinds),
            downloads=(
                f'<table class="files"><thead><tr><th>System</th><th>Kind</th><th>File</th>'
                f'<th class="num">Size</th></tr></thead><tbody>{rows}</tbody></table>'
                f'<p class="small">Checksums: <a href="{html.escape(assets["sha256"]["url"])}">SHA256SUMS.txt</a>'
                f' · <a href="{html.escape(release["url"])}">release page</a></p>'
            ),
        )
    else:
        values.update(
            version_line="First release coming soon",
            main_href="app/" if web else "#download",
            main_label="Run in the browser" if web else "Build from source",
            main_note="No installation · Chrome, Firefox, Safari, Edge" if web else "Rust and one command",
            kinds_json="{}",
            downloads=(
                '<p class="soon">The first packaged release is on its way: a Windows installer, a '
                'macOS disk image and Linux AppImage / .deb / .tar.gz will be listed here.</p>'
            ),
        )
    values["web_button"] = (
        '<a class="button secondary" href="app/">Run in the browser</a>' if web and release else ""
    )
    sections = changelog_sections(changelog_md, 3)
    values["changes"] = "\n".join(
        f'<article class="release"><h3>{html.escape(s["version"])} <span>{html.escape(s["date"])}</span></h3>'
        f"{markdown(s['body'])}</article>"
        for s in sections
    )
    return values


def build(out, release=None, web=None):
    theme = tomllib.loads((ROOT / "assets" / "theme.toml").read_text())
    values = page_values(release, web, (ROOT / "CHANGELOG.md").read_text())
    values["bg_dark"] = theme["bg_dark"]
    if out.exists():
        shutil.rmtree(out)
    shutil.copytree(SITE / "assets", out / "assets")
    (out / "assets" / "theme.css").write_text(theme_css(theme))
    for name in ["screenshot.png", "pattern-test.png", "motion-test.png", "pixel-refresh.png", "og.png",
                 "pixelprobe-mark.png"]:
        shutil.copy(ROOT / "docs" / "assets" / name, out / "assets" / name)
    shutil.copy(ROOT / "assets" / "icon.png", out / "assets" / "icon.png")
    if web:
        shutil.copytree(web, out / "app")
    (out / "index.html").write_text(render((SITE / "template.html").read_text(), values))
    (out / ".nojekyll").write_text("")
    return out / "index.html"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--release", type=Path, help="gh release view --json tagName,publishedAt,url,assets")
    ap.add_argument("--web", type=Path, help="the web build to serve at app/")
    a = ap.parse_args()
    release = json.loads(a.release.read_text()) if a.release else None
    print(build(a.out, release, a.web))


if __name__ == "__main__":
    main()
