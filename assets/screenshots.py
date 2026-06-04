#!/usr/bin/env python3
"""README screenshots, taken by the app itself over its control channel.

Starts pixelprobe with --control, runs commands, saves `ui.screenshot` PNGs into docs/assets and
scales the full-screen ones down to 1600 px wide. Run on a desktop session (the app opens a
window and goes full screen for the tests):

  cargo build --release -p pixelprobe
  python3 assets/screenshots.py [--bin target/release/pixelprobe] [--port 7879]
"""
import argparse
import json
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "docs" / "assets"
MAX_W = 1600

# (file, commands to run first, seconds to wait before the screenshot)
SHOTS = [
    ("screenshot.png", [], 1.0),
    ("pattern-test.png", ["test.pattern"], 1.5),
    ("motion-test.png", ["view.close", "test.motion"], 2.0),
    ("pixel-refresh.png", ["view.close", "test.pixel_refresh"], 1.5),
    ("help.png", ["view.close", "app.help"], 1.0),
]


class Control:
    def __init__(self, port):
        for _ in range(100):
            try:
                self.f = socket.create_connection(("127.0.0.1", port)).makefile("rw")
                return
            except OSError:
                time.sleep(0.1)
        sys.exit(f"no control channel on port {port}")

    def call(self, method, params=None):
        self.f.write(json.dumps({"id": 1, "method": method, "params": params or {}}) + "\n")
        self.f.flush()
        reply = json.loads(self.f.readline())
        if "error" in reply:
            sys.exit(f"{method}: {reply['error']}")
        return reply.get("ok")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bin", default=str(ROOT / "target" / "release" / "pixelprobe"))
    ap.add_argument("--port", type=int, default=7879)
    a = ap.parse_args()

    app = subprocess.Popen([a.bin, "--control", str(a.port)], stderr=subprocess.DEVNULL)
    try:
        ctl = Control(a.port)
        time.sleep(1.5)  # first frames, display list
        OUT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory() as tmp:
            for name, commands, wait in SHOTS:
                for c in commands:
                    ctl.call(c)
                time.sleep(wait)
                raw = Path(tmp) / name
                ctl.call("ui.screenshot", {"path": str(raw)})
                img = Image.open(raw).convert("RGB")
                if img.width > MAX_W:
                    img = img.resize((MAX_W, round(img.height * MAX_W / img.width)), Image.LANCZOS)
                img.save(OUT / name, optimize=True)
                print(f"docs/assets/{name} {img.width}x{img.height}")
        ctl.call("view.close")
    finally:
        app.terminate()
        app.wait(timeout=10)


if __name__ == "__main__":
    main()
