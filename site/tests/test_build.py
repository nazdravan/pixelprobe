"""Tests for site/build.py: python3 -m unittest discover -s site/tests"""
import copy
import json
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

SITE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(SITE))
sys.dont_write_bytecode = True
import build  # noqa: E402

RELEASE = json.loads((SITE / "tests" / "release.json").read_text())


class Build(unittest.TestCase):
    def build(self, release=None, web=None):
        tmp = Path(tempfile.mkdtemp())
        index = build.build(tmp / "_site", release, web)
        return tmp / "_site", index.read_text()

    def test_with_a_release_links_every_package(self):
        out, page = self.build(RELEASE)
        for a in RELEASE["assets"]:
            self.assertIn(a["url"], page)
        self.assertIn("Version 0.1.0 · 2026-11-01", page)
        self.assertNotIn("{{", page)
        self.assertTrue((out / "assets" / "theme.css").exists())
        self.assertTrue((out / ".nojekyll").exists())

    def test_download_button_has_one_package_per_system(self):
        _, page = self.build(RELEASE)
        start = page.index('data-kinds="') + len('data-kinds="')
        raw = page[start:page.index('"', start)].replace("&quot;", '"')
        kinds = json.loads(raw)
        self.assertEqual(set(kinds), {"windows", "mac", "linux"})
        self.assertTrue(kinds["mac"]["url"].endswith("macos-universal.dmg"))

    def test_a_missing_package_fails_the_build(self):
        broken = copy.deepcopy(RELEASE)
        broken["assets"] = [a for a in broken["assets"] if not a["name"].endswith(".msi")]
        with self.assertRaises(SystemExit):
            self.build(broken)

    def test_without_a_release_the_page_says_so(self):
        _, page = self.build(None)
        self.assertIn("First release coming soon", page)
        self.assertIn("first packaged release is on its way", page)
        self.assertNotIn("releases/download", page)

    def test_theme_css_has_every_theme_colour(self):
        out, _ = self.build(None)
        css = (out / "assets" / "theme.css").read_text()
        theme = tomllib.loads((SITE.parent / "assets" / "theme.toml").read_text())
        for key in build.THEME_KEYS:
            self.assertIn(f"--{key.replace('_', '-')}: {theme[key]};", css)
        for c in theme["bars"]:
            self.assertIn(c, css)

    def test_style_uses_no_colour_outside_the_theme(self):
        import re
        css = (SITE / "assets" / "style.css").read_text()
        self.assertEqual(re.findall(r"#[0-9a-fA-F]{3,8}\b", css), [])

    def test_changelog_markdown(self):
        html = build.markdown("### Added\n\n- one `code`\n  continued\n- **two** [x](https://e.org)\n")
        self.assertIn("<h4>Added</h4>", html)
        self.assertIn("<li>one <code>code</code> continued</li>", html)
        self.assertIn('<strong>two</strong> <a href="https://e.org">x</a>', html)

    def test_escaping(self):
        self.assertEqual(build.inline("<b> & `<i>`"), "&lt;b&gt; &amp; <code>&lt;i&gt;</code>")


if __name__ == "__main__":
    unittest.main()
