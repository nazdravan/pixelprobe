#!/usr/bin/env python3
"""pixelprobe icon: a monitor showing SMPTE colour bars, with a magnifier over its pixels.

All colours come from assets/theme.toml. Drawn at 1024 px with 4x supersampling, then
scaled down. Writes:

  assets/icon.png               256x256, window icon and favicon
  assets/pixelprobe.ico         16...256, Windows
  assets/pixelprobe.icns        16...1024, badge at 824/1024 of the square (Apple grid), macOS
  docs/assets/pixelprobe-mark.png  512x512, README header

Usage: python3 assets/icon.py [--sheet out.png]   (--sheet: preview at 16, 32, 64, 256 px)
"""
import argparse
import shutil
import subprocess
import tempfile
import tomllib
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
THEME = tomllib.loads((ROOT / "assets" / "theme.toml").read_text())
N = 1024
SS = 4  # supersampling factor


def rgb(name):
    s = THEME[name].lstrip("#")
    return tuple(int(s[i:i + 2], 16) for i in (0, 2, 4))


def hexrgb(s):
    s = s.lstrip("#")
    return tuple(int(s[i:i + 2], 16) for i in (0, 2, 4))


def badge(size):
    """The icon: rounded badge with the monitor and, above 32 px, the magnifier."""
    W = size * SS
    u = W / 1000  # drawing unit: the badge is 1000 x 1000
    img = Image.new("RGBA", (W, W), (0, 0, 0, 0))

    # Badge: vertical gradient inside a rounded square.
    top, bottom = rgb("badge_top"), rgb("badge_bottom")
    grad = Image.new("RGBA", (W, W))
    gd = ImageDraw.Draw(grad)
    for y in range(W):
        t = y / (W - 1)
        gd.line([(0, y), (W, y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(top, bottom)) + (255,))
    mask = Image.new("L", (W, W), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, W - 1, W - 1], round(225 * u), fill=255)
    img.paste(grad, (0, 0), mask)
    d = ImageDraw.Draw(img)

    bezel, black = rgb("bezel"), rgb("bg_dark")
    bars = [hexrgb(c) for c in THEME["bars"]]

    # Stand: neck and foot.
    d.rectangle([450 * u, 690 * u, 550 * u, 790 * u], fill=bezel)
    d.rounded_rectangle([300 * u, 775 * u, 700 * u, 825 * u], round(25 * u), fill=bezel)

    # Monitor: bezel, then the screen.
    sx0, sy0, sx1, sy1 = 130 * u, 175 * u, 870 * u, 700 * u
    d.rounded_rectangle([sx0, sy0, sx1, sy1], round(45 * u), fill=bezel)
    x0, y0, x1, y1 = sx0 + 30 * u, sy0 + 30 * u, sx1 - 30 * u, sy1 - 30 * u
    screen = Image.new("RGBA", (round(x1 - x0), round(y1 - y0)), black + (255,))
    sd = ImageDraw.Draw(screen)
    sw, sh = screen.size
    split = round(sh * 0.72)
    for i, c in enumerate(bars):
        a, b = round(i * sw / 7), round((i + 1) * sw / 7)
        sd.rectangle([a, 0, b, split], fill=c)
    # Bottom row: reverse bars (blue, black, magenta, black, cyan, black, white).
    reverse = [bars[6], black, bars[4], black, bars[2], black, bars[0]]
    for i, c in enumerate(reverse):
        a, b = round(i * sw / 7), round((i + 1) * sw / 7)
        sd.rectangle([a, split + round(sh * 0.04), b, sh], fill=c)
    smask = Image.new("L", screen.size, 0)
    ImageDraw.Draw(smask).rounded_rectangle([0, 0, sw - 1, sh - 1], round(18 * u), fill=255)
    img.paste(screen, (round(x0), round(y0)), smask)

    # Magnifier over the lower right: the lens shows R, G, B sub-pixel stripes.
    if size <= 32:
        return img.resize((size, size), Image.LANCZOS)
    cx, cy, r = 670 * u, 545 * u, 150 * u
    ring = rgb("ring")
    d.line([(cx + r * 0.62, cy + r * 0.62), (cx + r * 1.5, cy + r * 1.5)], fill=ring, width=round(70 * u))
    d.ellipse([cx - r - 32 * u, cy - r - 32 * u, cx + r + 32 * u, cy + r + 32 * u], fill=ring)
    lens = Image.new("RGBA", (round(2 * r), round(2 * r)), black + (255,))
    ld = ImageDraw.Draw(lens)
    stripe = 2 * r / 6
    for i, c in enumerate([(255, 0, 0), (0, 255, 0), (0, 0, 255)] * 2):
        ld.rectangle([i * stripe + 6 * u, 0, (i + 1) * stripe - 6 * u, 2 * r], fill=c)
    lmask = Image.new("L", lens.size, 0)
    ImageDraw.Draw(lmask).ellipse([0, 0, 2 * r - 1, 2 * r - 1], fill=255)
    img.paste(lens, (round(cx - r), round(cy - r)), lmask)

    return img.resize((size, size), Image.LANCZOS)


def apple_grid(size):
    """macOS: the badge fills 824/1024 of the square, the rest is transparent."""
    out = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    inner = max(1, round(size * 824 / 1024))
    b = badge(inner)
    out.paste(b, ((size - inner) // 2, (size - inner) // 2), b)
    return out


def write_icns(path):
    """With Apple's iconutil (macOS) every size gets its own frame; elsewhere Pillow writes
    the file but leaves out the 16 and 32 px frames, which macOS then scales down."""
    if shutil.which("iconutil"):
        with tempfile.TemporaryDirectory() as tmp:
            iconset = Path(tmp) / "pixelprobe.iconset"
            iconset.mkdir()
            for s in (16, 32, 128, 256, 512):
                apple_grid(s).save(iconset / f"icon_{s}x{s}.png")
                apple_grid(2 * s).save(iconset / f"icon_{s}x{s}@2x.png")
            subprocess.run(["iconutil", "-c", "icns", "-o", str(path), str(iconset)], check=True)
    else:
        mac = [apple_grid(s) for s in (16, 32, 64, 128, 256, 512, 1024)]
        mac[-1].save(path, append_images=mac[:-1])


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--sheet", help="also write a preview sheet at 16, 32, 64 and 256 px")
    a = ap.parse_args()

    full = badge(N)
    full.resize((256, 256), Image.LANCZOS).save(ROOT / "assets" / "icon.png", optimize=True)
    # Pillow's .ico writer scales one image, so build each size and save them together.
    ico = [badge(s) for s in (16, 24, 32, 48, 64, 128, 256)]
    ico[-1].save(ROOT / "assets" / "pixelprobe.ico", sizes=[i.size for i in ico], append_images=ico[:-1])
    write_icns(ROOT / "assets" / "pixelprobe.icns")
    (ROOT / "docs" / "assets").mkdir(parents=True, exist_ok=True)
    full.resize((512, 512), Image.LANCZOS).save(ROOT / "docs" / "assets" / "pixelprobe-mark.png", optimize=True)

    if a.sheet:
        sizes = [16, 32, 64, 256]
        sheet = Image.new("RGBA", (sum(sizes) + 40 * len(sizes) + 40, 256 + 80), rgb("bg_light") + (255,))
        x = 40
        for s in sizes:
            sheet.paste(badge(s), (x, 40), badge(s))
            x += s + 40
        dark = Image.new("RGBA", sheet.size, rgb("bg_dark") + (255,))
        x = 40
        for s in sizes:
            dark.paste(badge(s), (x, 40), badge(s))
            x += s + 40
        both = Image.new("RGBA", (sheet.width, sheet.height * 2))
        both.paste(sheet, (0, 0))
        both.paste(dark, (0, sheet.height))
        both.save(a.sheet)
    print("assets/icon.png, assets/pixelprobe.ico, assets/pixelprobe.icns, docs/assets/pixelprobe-mark.png")


if __name__ == "__main__":
    main()
