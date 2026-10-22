"""Synthetic browser integration checks; no model service is used."""
import os
import json
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
        for name, sample in [("compact", "static-components"), ("raw", "static-components-raw"), ("spec", "static-components")]:
            source = self.root / f"{name}.html"
            arguments = ["--spec", ROOT / "samples" / sample / "deck.json"] if name == "spec" else ["-d", ROOT / "samples" / sample]
            subprocess.run([self.binary, "export", *arguments, "-o", source], check=True)
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
            pdf_info = subprocess.run(["pdfinfo", self.root / name / "deck.pdf"],
                                      check=True, capture_output=True, text=True, timeout=15,
                                      env={**os.environ, "LC_ALL": "C"})
            pages = next(line for line in pdf_info.stdout.splitlines() if line.startswith("Pages:"))
            self.assertEqual(int(pages.split(":", 1)[1].strip()), 2)
            outputs.append(manifest)
        for other in ["raw", "spec"]:
            for index in range(1, 3):
                filename = f"slide-{index:03}.png"
                with Image.open(self.root / "compact" / filename) as compact, Image.open(self.root / other / filename) as raw:
                    diff = ImageChops.difference(compact.convert("RGB"), raw.convert("RGB"))
                    # Permit only the existing narrow glyph-edge raster noise.
                    self.assertLessEqual(max(high for _, high in diff.getextrema()), 1)
                    self.assertLessEqual(sum(pixel != (0, 0, 0) for pixel in diff.get_flattened_data()), 32)
            self.assertEqual((self.root / "compact.html").read_bytes(), (self.root / f"{other}.html").read_bytes())
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

    def test_spec_errors_preserve_output_and_skip_local_code(self):
        sample = ROOT / "samples/static-components/deck.json"
        data = json.loads(sample.read_text(encoding="utf-8"))
        source, output = self.root / "request.json", self.root / "out.html"
        output.write_text("previous output")
        data["slides"][0]["blocks"][0]["columns"][0]["blocks"][0]["paragraphs"][0] = 42
        source.write_text(json.dumps(data), encoding="utf-8")
        result = subprocess.run([self.binary, "export", "--spec", source, "-o", output], capture_output=True, text=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn("$.slides[0].blocks[0].columns[0].blocks[0].paragraphs[0]: expected string", result.stderr)
        self.assertEqual(output.read_text(), "previous output")
        self.assertEqual(sorted(p.name for p in self.root.iterdir()), ["out.html", "request.json"])
        for folder, filename in [("layouts", "slide-feature.html"), ("styles", "evil.css")]:
            directory = self.root / folder
            directory.mkdir()
            (directory / filename).write_text("LOCAL_CODE_MUST_NOT_LOAD")
        (self.root / "layouts/slide-feature.js").write_text("throw Error('LOCAL_JS_MUST_NOT_LOAD')")
        data = json.loads(sample.read_text(encoding="utf-8"))
        data["title"] = '</title><script>SPEC_EXECUTED</script>'
        data["slides"][0]["heading"] = '<script>SPEC_EXECUTED</script>'
        data["slides"][0]["blocks"] = [{"type":"text","text":"한글 **literal** {{heading}} <script>SPEC_EXECUTED</script>"}]
        source.write_text(json.dumps(data), encoding="utf-8")
        subprocess.run([self.binary, "export", "--spec", source, "-o", output], cwd=self.root, check=True)
        exported = output.read_text(encoding="utf-8")
        self.assertNotIn("LOCAL_CODE", exported)
        self.assertNotIn("LOCAL_JS", exported)
        self.assertNotIn("<script>SPEC_EXECUTED", exported)
        self.assertIn("&lt;/title&gt;", exported)
        manifest = self.run_render(output)
        self.assertIn("한글 **literal** {{heading}} <script>SPEC_EXECUTED</script>", manifest["slides"][0]["text"])

    def test_markup_preset_themes_apply(self):
        corners = {}
        for theme in ["default", "night"]:
            output = self.root / f"{theme}.html"
            subprocess.run([self.binary, "export", "--markup", ROOT / "samples/markup/deck.html",
                            "--theme", theme, "-o", output], check=True)
            manifest = self.run_render(output, theme)
            self.assertEqual(manifest["slide_count"], 5)
            self.assertIn("안녕하세요", manifest["slides"][0]["text"])
            self.assertIn("가상 시범 운영 검토", manifest["slides"][1]["text"])
            self.assertTrue(all("합성 자료" in s["text"] for s in manifest["slides"]))
            self.assertIn("기능 확인용 합성 자료", manifest["slides"][1]["text"])
            self.assertIn("카드와 단 나누기", manifest["slides"][2]["text"])
            self.assertIn("라벨이 붙은 패널", manifest["slides"][2]["text"])
            self.assertIn("준비 6주", manifest["slides"][3]["text"])
            # The local SVG is embedded and painted: its solid #2563EB box shows up.
            flow = Image.open(self.root / theme / manifest["slides"][4]["file"]).convert("RGB")
            self.assertTrue(any(c == (0x25, 0x63, 0xEB) for _, c in flow.getcolors(1 << 22)))
            self.assertFalse(any(s["overflow"] for s in manifest["slides"]))
            # Bare <wb-slide>: ink is centered on the canvas.
            bare = Image.open(self.root / theme / manifest["slides"][0]["file"]).convert("RGB")
            ink = ImageChops.difference(bare, Image.new("RGB", bare.size, bare.getpixel((4, 4)))).crop((0, 0, 1900, 960)).getbbox()
            self.assertLess(abs((ink[0] + ink[2]) / 2 - 960), 40)
            self.assertLess(abs((ink[1] + ink[3]) / 2 - 540), 40)
            # Preset --wb-center-size (40px+) applies: glyphs are far taller than unstyled 16px text.
            self.assertGreater(ink[3] - ink[1], 60)
            png = Image.open(self.root / theme / manifest["slides"][1]["file"]).convert("RGB")
            corners[theme] = png.getpixel((4, 4))
        self.assertEqual(corners["default"], (255, 255, 255))
        self.assertEqual(corners["night"], (0x11, 0x18, 0x27))


if __name__ == "__main__":
    unittest.main()
