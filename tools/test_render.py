"""Synthetic browser integration checks; no model service is used."""
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import unittest

from PIL import Image, ImageChops

from render import render

ROOT = Path(__file__).resolve().parents[1]


class RenderTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if not os.environ.get("WB_RENDER_FONT"):
            raise RuntimeError("set WB_RENDER_FONT to a licensed Korean variable TTF")
        cls.font = Path(os.environ["WB_RENDER_FONT"])
        cls.browser = os.environ.get("WB_RENDER_BROWSER")
        cls.binary = Path(os.environ.get("WB_SLIDE_BIN", ROOT / "target/debug/wb-slide"))

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def html(self, body, script=""):
        path = self.root / "input.html"
        path.write_text('<!doctype html><html lang="ko"><meta charset="utf-8">'
                        '<style>body{margin:0}.ms-slide-container{width:960px;height:540px}</style>'
                        + body + script + '</html>', encoding="utf-8")
        return path

    def run_render(self, source, name="out", pdf=False):
        return render(source, self.root / name, self.font, self.browser, pdf=pdf)

    def test_korean_raw_and_compact_decks_match(self):
        outputs = []
        for name, sample in [("compact", "static-components"), ("raw", "static-components-raw")]:
            source = self.root / f"{name}.html"
            subprocess.run([self.binary, "export", "-d", ROOT / "samples" / sample, "-o", source], check=True)
            manifest = self.run_render(source, name, pdf=True)
            self.assertEqual(manifest["slide_count"], 2)
            self.assertIn("시범 운영", manifest["slides"][0]["text"])
            self.assertIn("240명", manifest["slides"][1]["text"])
            for slide in manifest["slides"]:
                self.assertFalse(slide["overflow"])
                self.assertTrue(slide["fonts"])
                self.assertTrue(all(f["isCustomFont"] for f in slide["fonts"]))
                png = (self.root / name / slide["file"]).read_bytes()
                self.assertEqual(struct.unpack(">II", png[16:24]), (1920, 1080))
            self.assertTrue((self.root / name / "deck.pdf").read_bytes().startswith(b"%PDF-"))
            outputs.append(manifest)
        for index in range(1, 3):
            filename = f"slide-{index:03}.png"
            with Image.open(self.root / "compact" / filename) as compact, Image.open(self.root / "raw" / filename) as raw:
                diff = ImageChops.difference(compact.convert("RGB"), raw.convert("RGB"))
                # Chromium can vary one color level at a handful of glyph
                # edge pixels between contexts; do not tolerate layout drift.
                self.assertLessEqual(max(high for _, high in diff.getextrema()), 1)
                self.assertLessEqual(sum(pixel != (0, 0, 0) for pixel in diff.get_flattened_data()), 32)
        self.assertEqual((self.root / "compact.html").read_bytes(), (self.root / "raw.html").read_bytes())
        artifacts = os.environ.get("WB_RENDER_ARTIFACTS")
        if artifacts:
            shutil.copytree(self.root / "compact", artifacts)

    def assert_failed_without_output(self, source, message):
        with self.assertRaisesRegex(ValueError, message):
            self.run_render(source)
        self.assertFalse((self.root / "out").exists())

    def test_javascript_failure_cannot_be_a_successful_blank_deck(self):
        self.assert_failed_without_output(self.html('<div class="ms-slide-container">본문</div>',
            '<script>class Legacy extends SlideBase {}</script>'), "SlideBase")

    def test_explicit_font_policy_overrides_inline_important_font(self):
        source = self.html('<div class="ms-slide-container" style="font-family:serif!important">본문 123</div>')
        manifest = self.run_render(source)
        self.assertTrue(manifest["slides"][0]["fonts"])
        self.assertTrue(all(font["isCustomFont"] for font in manifest["slides"][0]["fonts"]))

    def test_missing_and_network_assets_fail(self):
        for url in ["missing.png", "https://example.invalid/missing.png"]:
            with self.subTest(url=url):
                self.assert_failed_without_output(self.html(
                    f'<div class="ms-slide-container">본문<img src="{url}"></div>'), "asset|browser errors")

    def test_missing_glyph_fails_before_delivery(self):
        self.assert_failed_without_output(self.html('<div class="ms-slide-container">본문\U0010FFFF</div>'),
                                          "font lacks")

    def test_empty_deck_and_unresolved_components_fail(self):
        self.assert_failed_without_output(self.html('<p>Not a deck</p>'), "containers")
        self.assert_failed_without_output(self.html('<div class="ms-slide-container"><slide-x>본문</slide-x></div>'),
                                          "unresolved custom")

    def test_existing_directory_is_preserved(self):
        existing = self.root / "out"
        existing.mkdir()
        (existing / "keep.txt").write_text("keep")
        with self.assertRaisesRegex(ValueError, "already exists"):
            self.run_render(self.html('<div class="ms-slide-container">본문</div>'))
        self.assertEqual((existing / "keep.txt").read_text(), "keep")


if __name__ == "__main__":
    unittest.main()
