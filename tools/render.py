"""Reviewed, optional renderer for trusted, static wb-slide HTML (Python 3.11+)."""
import argparse
import base64
import hashlib
from importlib.metadata import version
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unicodedata

from fontTools.ttLib import TTFont
from playwright.sync_api import sync_playwright

VIEWPORT = {"width": 960, "height": 540}
TIMEOUT = 15_000


def digest(data):
    return hashlib.sha256(data).hexdigest()


def font_policy(path):
    data = path.read_bytes()
    if path.suffix.lower() != ".ttf" or not 0 < len(data) <= 32 * 1024 * 1024:
        raise ValueError("use a licensed variable .ttf font (at most 32 MiB)")
    with TTFont(path) as font:
        axes = {a.axisTag: a for a in font["fvar"].axes} if "fvar" in font else {}
        if "wght" not in axes:
            raise ValueError("this renderer requires a variable TTF with a wght axis")
        axis = axes["wght"]
        coverage = {cp for cp, glyph in font.getBestCmap().items() if glyph != ".notdef"}
        policy = {"file": path.name, "sha256": digest(data),
                  "family": font["name"].getDebugName(16) or font["name"].getDebugName(1),
                  "weight_min": axis.minValue, "weight_max": axis.maxValue}
    return data, coverage, policy


def require_coverage(text, weights, coverage, policy):
    missing = sorted({ord(c) for c in text if not c.isspace()
                      and unicodedata.category(c) not in {"Cc", "Cf"}} - coverage)
    if missing:
        raise ValueError("font lacks visible characters: " + ", ".join(f"U+{cp:04X}" for cp in missing[:16]))
    if any(not policy["weight_min"] <= float(w) <= policy["weight_max"] for w in weights):
        raise ValueError("font does not cover the slide's requested weights")


def ready(page):
    page.evaluate("""async () => {
      const active = document.querySelector('.ms-slide-container.active');
      const weights = [...new Set([active,...active.querySelectorAll('*')]
        .map(e => getComputedStyle(e).fontWeight))];
      await Promise.race([
        Promise.all([...weights.map(w => document.fonts.load(w+' 16px wb-render', active.innerText)),
          document.fonts.ready, ...[...document.images].map(i => i.decode())]),
        new Promise((_, reject) => setTimeout(() => reject(Error('asset readiness timeout')), 15000))
      ]);
      if (![...document.fonts].some(f => f.family === 'wb-render' && f.status === 'loaded'))
        throw Error('required render font failed to load');
      await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    }""")


# Layout problems for the repair loop: outermost overflow, clipped content, overlapping text/images.
FINDINGS = """(s, slide) => {
  const b=s.getBoundingClientRect(), box=e=>e.getBoundingClientRect();
  const vis=[...s.querySelectorAll('*')].filter(e=>{const r=box(e); return r.width && r.height});
  const id=e=>({element:e.tagName+'.'+(e.getAttribute('class')??''),
    text:(e.alt||e.innerText||e.textContent||'').trim().slice(0,60)});
  const find=(kind,e,extra)=>({slide,kind,...id(e),...extra});
  const out=vis.filter(e=>{const r=box(e); return r.left<b.left-1 || r.top<b.top-1 ||
    r.right>b.right+1 || r.bottom>b.bottom+1});
  const clipped=vis.filter(e=>{const c=getComputedStyle(e);
    return (c.overflowX!=='visible' || c.overflowY!=='visible') &&
      (e.scrollHeight>e.clientHeight+1 || e.scrollWidth>e.clientWidth+1)});
  // Block-level text boxes and images only: a wrapped inline's box is the union of its lines.
  // Slide chrome and layered covers overlap by design.
  const leaves=vis.filter(e=>(e.localName==='img' || [...e.childNodes].some(n=>n.nodeType===3 && n.textContent.trim())) &&
    getComputedStyle(e).display!=='inline' && !e.closest('.ms-watermark, .ms-footer-logo, .ms-page-number'));
  const layered=(a,c)=>a.closest('.cover-stack') && a.closest('.cover-stack')===c.closest('.cover-stack');
  // ponytail: O(n²) pairwise overlap; fine for one slide's leaves, add a sweep line if decks get dense.
  const overlap=[];
  leaves.forEach((a,i)=>leaves.slice(i+1).forEach(c=>{const p=box(a), q=box(c);
    if (!a.contains(c) && !c.contains(a) && !layered(a,c) && Math.min(p.right,q.right)-Math.max(p.left,q.left)>2 &&
        Math.min(p.bottom,q.bottom)-Math.max(p.top,q.top)>2) overlap.push(find('overlap',a,{with:id(c)}))}));
  return [...out.filter(e=>!out.some(a=>a!==e && a.contains(e))).map(e=>find('overflow',e)),
    ...clipped.map(e=>find('clipped',e)), ...overlap];
}"""


def check_errors(errors):
    if errors:
        raise ValueError("browser errors: " + "; ".join(errors[:12]))


def render(html_path, output, font_path, browser_path=None, scale=2, pdf=False):
    html_path, output, font_path = html_path.resolve(), output.absolute(), font_path.resolve()
    if output.exists():
        raise ValueError("output directory already exists; choose a new directory")
    html_bytes = html_path.read_bytes()
    font_bytes, coverage, policy = font_policy(font_path)
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".wb-render-", dir=output.parent))
    try:
        with sync_playwright() as playwright:
            launch = {"headless": True}
            if browser_path:
                launch["executable_path"] = str(Path(browser_path).resolve())
            browser = playwright.chromium.launch(**launch)
            context = browser.new_context(viewport=VIEWPORT, device_scale_factor=scale,
                                          locale="ko-KR", timezone_id="Asia/Seoul",
                                          color_scheme="light", reduced_motion="reduce",
                                          service_workers="block", accept_downloads=False)
            page = context.new_page()
            page.set_default_timeout(TIMEOUT)
            errors = []
            page.on("pageerror", lambda error: errors.append(str(error)))
            page.on("console", lambda message: errors.append(message.text) if message.type == "error" else None)
            page.on("requestfailed", lambda request: errors.append(f"asset failed: {request.url}"))

            def local_only(route):
                if route.request.url.startswith(("file:", "data:")):
                    route.continue_()
                else:
                    errors.append(f"network asset blocked: {route.request.url}")
                    route.abort()

            page.route("**/*", local_only)
            page.goto(html_path.as_uri(), wait_until="load", timeout=TIMEOUT)
            check_errors(errors)
            page.add_style_tag(content=f"""
              @font-face {{ font-family: 'wb-render';
                src: url(data:font/ttf;base64,{base64.b64encode(font_bytes).decode()});
                font-weight: {policy['weight_min']} {policy['weight_max']}; }}
              .ms-slide-container, .ms-slide-container * {{
                font-family: 'wb-render' !important; font-synthesis: none !important;
              }}
              *, *::before, *::after {{ animation: none !important; transition: none !important;
                -webkit-print-color-adjust: exact !important; print-color-adjust: exact !important; }}
            """)
            # Inline !important declarations outrank stylesheet rules. Apply
            # the explicit preview font at the same origin on each element.
            page.evaluate("""() => document.querySelectorAll('.ms-slide-container, .ms-slide-container *')
              .forEach(e => e.style.setProperty('font-family', 'wb-render', 'important'))""")
            slides = page.locator(".ms-slide-container")
            count = slides.count()
            if not 0 < count <= 200:
                raise ValueError("expected 1–200 static wb-slide containers")
            custom = page.evaluate("[...document.querySelectorAll('*')].filter(e=>e.localName.includes('-')).map(e=>e.localName)")
            if custom:
                raise ValueError("static HTML required; unresolved custom tags: " + ", ".join(sorted(set(custom))))
            cdp = context.new_cdp_session(page)
            cdp.send("DOM.enable")
            cdp.send("CSS.enable")
            records = []
            for index in range(count):
                page.evaluate("""i => document.querySelectorAll('.ms-slide-container').forEach((s,n)=>
                    s.classList.toggle('active', n===i))""", index)
                ready(page)
                metrics = slides.nth(index).evaluate("""s => {
                  const b=s.getBoundingClientRect();
                  return {text:s.innerText, notes:s.querySelector('.wb-notes[hidden]')?.textContent ?? '', width:b.width, height:b.height,
                    weights:[...new Set([s,...s.querySelectorAll('*')].map(e=>getComputedStyle(e).fontWeight))],
                    overflow:[...s.querySelectorAll('*')].filter(e=>{const r=e.getBoundingClientRect();
                      return r.width && r.height && (r.left<b.left-1 || r.top<b.top-1 ||
                        r.right>b.right+1 || r.bottom>b.bottom+1)}).map(e=>e.tagName+'.'+e.className)};
                }""")
                metrics["findings"] = slides.nth(index).evaluate(FINDINGS, index+1)
                if abs(metrics["width"] - 960) > 1 or abs(metrics["height"] - 540) > 1:
                    raise ValueError(f"slide {index+1}: expected a 960x540 canvas")
                require_coverage(metrics["text"], metrics["weights"], coverage, policy)
                document = cdp.send("DOM.getDocument")
                nodes = cdp.send("DOM.querySelectorAll", {"nodeId": document["root"]["nodeId"],
                                 "selector": ".ms-slide-container.active, .ms-slide-container.active *"})
                fonts = {}
                for node in nodes["nodeIds"]:
                    for font in cdp.send("CSS.getPlatformFontsForNode", {"nodeId": node})["fonts"]:
                        if not font["isCustomFont"] and font["glyphCount"]:
                            raise ValueError(f"slide {index+1}: undeclared font fallback: {font['familyName']}")
                        fonts[font["postScriptName"]] = {k: v for k, v in font.items() if k != "glyphCount"}
                filename = f"slide-{index+1:03}.png"
                slides.nth(index).screenshot(path=str(stage / filename), animations="disabled")
                check_errors(errors)
                records.append({"index": index+1, "file": filename, "text": metrics["text"],
                                "notes": metrics["notes"],
                                "sha256": digest((stage / filename).read_bytes()),
                                "overflow": metrics["overflow"], "findings": metrics["findings"], "fonts": list(fonts.values())})
            if pdf:
                page.pdf(path=str(stage / "deck.pdf"), prefer_css_page_size=True, print_background=True)
            check_errors(errors)
            manifest = {"version": 1, "input_sha256": digest(html_bytes), "browser": browser.version,
                        "renderer_sha256": digest(Path(__file__).read_bytes()),
                        "playwright": version("playwright"), "fonttools": version("fonttools"),
                        "viewport": VIEWPORT, "scale": scale,
                        "locale": "ko-KR", "timezone": "Asia/Seoul", "font": policy,
                        "slide_count": count, "pdf": "deck.pdf" if pdf else None,
                        "overflow_check": "element bounds only; visual review still required", "slides": records}
            (stage / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            browser.close()
        os.rename(stage, output)
        return manifest
    finally:
        if stage.exists():
            shutil.rmtree(stage)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("html", type=Path)
    parser.add_argument("--output", required=True, type=Path, help="new directory; never overwrites")
    parser.add_argument("--font-file", required=True, type=Path, help="licensed Korean variable TTF")
    parser.add_argument("--browser", type=Path, help="optional installed Chromium/Chrome executable")
    parser.add_argument("--scale", choices=[1, 2], type=int, default=2)
    parser.add_argument("--pdf", action="store_true")
    parser.add_argument("--check", action="store_true", help="print layout findings as JSON; exit 3 if any")
    args = parser.parse_args()
    try:
        manifest = render(args.html, args.output, args.font_file, args.browser, args.scale, args.pdf)
    except Exception as error:
        print(f"render failed: {error}", file=sys.stderr)
        return 1
    if args.check:
        findings = [f for slide in manifest["slides"] for f in slide["findings"]]
        print(json.dumps(findings, ensure_ascii=False))
        return 3 if findings else 0
    print(f"Rendered {manifest['slide_count']} slides to {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
