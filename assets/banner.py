#!/usr/bin/env python3
"""README banner and social-preview image, in the app's own look: a dark monitor screen with
the logo, the name and a strip of SMPTE colour bars along the bottom.

Colours come from assets/theme.toml, the font is Inter (assets/fonts, SIL OFL), the logo is
drawn by assets/icon.py. The screen is opaque and its corners transparent, so it reads the same
on light and dark pages.

Usage:
  python3 assets/banner.py                      # docs/assets/pixelprobe-banner.png (2430x420)
  python3 assets/banner.py --og out.png         # also a 1200x630 social-preview image
"""
import argparse
import sys
import tomllib
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
sys.dont_write_bytecode = True  # importing icon.py must not leave __pycache__ in assets/
sys.path.insert(0, str(ROOT / "assets"))
import icon  # noqa: E402  (same folder: the logo)

THEME = tomllib.loads((ROOT / "assets" / "theme.toml").read_text())
FONTS = ROOT / "assets" / "fonts"
TAGLINE = "Monitor test utility · display info, motion, patterns, pixel refresh"
SS = 2  # supersampling


def hexrgb(s):
    s = s.lstrip("#")
    return tuple(int(s[i:i + 2], 16) for i in (0, 2, 4))


def mix(a, b, t):
    return tuple(round(x + (y - x) * t) for x, y in zip(a, b))


def screen(w, h, radius):
    """Dark rounded screen with the bezel line and the bars strip; transparent outside."""
    top, bottom = hexrgb(THEME["badge_top"]), hexrgb(THEME["badge_bottom"])
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    grad = Image.new("RGBA", (w, h))
    gd = ImageDraw.Draw(grad)
    for y in range(h):
        gd.line([(0, y), (w, y)], fill=mix(top, bottom, y / (h - 1)) + (255,))
    # Bars along the bottom: the seven bars, then the reverse row (as in the logo).
    bars = [hexrgb(c) for c in THEME["bars"]]
    black = hexrgb(THEME["bg_dark"])
    strip, rev = round(h * 0.075), round(h * 0.03)
    for i, c in enumerate(bars):
        gd.rectangle([round(i * w / 7), h - strip - rev, round((i + 1) * w / 7), h - rev], fill=c)
    for i, c in enumerate([bars[6], black, bars[4], black, bars[2], black, bars[0]]):
        gd.rectangle([round(i * w / 7), h - rev, round((i + 1) * w / 7), h], fill=c)
    mask = Image.new("L", (w, h), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, w - 1, h - 1], radius, fill=255)
    img.paste(grad, (0, 0), mask)
    ImageDraw.Draw(img).rounded_rectangle(
        [0, 0, w - 1, h - 1], radius, outline=hexrgb(THEME["bezel"]) + (255,), width=max(2, w // 400)
    )
    return img


def draw(w, h, name_px, tag_px, logo_px, pad, tagline=True):
    W, H = w * SS, h * SS
    img = screen(W, H, round(min(W, H) * 0.11))
    d = ImageDraw.Draw(img)
    fg, accent = hexrgb(THEME["fg_dark"]), hexrgb(THEME["accent_light"])
    usable = H - round(H * 0.105)  # above the bars strip

    logo = icon.badge(logo_px * SS)
    bold = ImageFont.truetype(str(FONTS / "Inter-Bold.ttf"), name_px * SS)
    regular = ImageFont.truetype(str(FONTS / "Inter-Regular.ttf"), tag_px * SS)
    # Logo and text as one block, centred horizontally.
    text_w = max(d.textlength("pixelprobe", font=bold), d.textlength(TAGLINE, font=regular) if tagline else 0)
    space = round(pad * 0.7 * SS)
    lx = round((W - (logo.width + space + text_w)) / 2)
    ly = (usable - logo.height) // 2
    img.paste(logo, (lx, ly), logo)
    x = lx + logo.width + space
    # "pixel" in the text colour, "probe" in the accent.
    name_box = d.textbbox((0, 0), "pixelprobe", font=bold, anchor="ls")
    tag_box = d.textbbox((0, 0), TAGLINE, font=regular, anchor="ls") if tagline else (0, 0, 0, 0)
    gap = round(name_px * 0.32 * SS) if tagline else 0
    block = (name_box[3] - name_box[1]) + gap + (tag_box[3] - tag_box[1])
    base = (usable - block) // 2 - name_box[1]
    d.text((x, base), "pixel", font=bold, fill=fg, anchor="ls")
    d.text((x + d.textlength("pixel", font=bold), base), "probe", font=bold, fill=accent, anchor="ls")
    if tagline:
        d.text((x + SS * 4, base + gap - tag_box[1]), TAGLINE, font=regular, fill=mix(fg, hexrgb(THEME["bg_dark"]), 0.3), anchor="ls")
    return img.resize((w, h), Image.LANCZOS)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("-o", "--out", default=str(ROOT / "docs" / "assets" / "pixelprobe-banner.png"))
    ap.add_argument("--og", help="also write a 1200x630 social-preview image here")
    a = ap.parse_args()
    draw(2430, 420, name_px=150, tag_px=50, logo_px=290, pad=70).save(a.out, optimize=True)
    print(a.out)
    if a.og:
        draw(1200, 630, name_px=130, tag_px=34, logo_px=250, pad=60, tagline=False).save(a.og, optimize=True)
        print(a.og)


if __name__ == "__main__":
    main()
