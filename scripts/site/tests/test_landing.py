"""Tests of scripts/site/landing.py (python3 -m unittest discover -s
scripts/site/tests -p 'test_*.py')."""

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

import landing  # noqa: E402
from landing import (TABLES, RenderError, build, check, load,  # noqa: E402
                     page_values, render, table_hash, texts)

EN = {"code": "en", "name": "English"}
ES = {"code": "es", "name": "Español"}


def read(path):
    with open(path, encoding="utf-8") as fh:
        return fh.read()


def master_assets():
    """The download patterns of the landing page before it became a template."""
    try:
        old = subprocess.run(["git", "show", "eb9fff0:site/index.html"], cwd=landing.ROOT,
                             capture_output=True, text=True, check=True).stdout
    except (OSError, subprocess.CalledProcessError):
        return None
    return re.findall(r'data-asset="([^"]*)"', old)


class RenderTest(unittest.TestCase):
    def test_render_replaces_text_and_value(self):
        out = render("<p>{{ index.pitch }}</p><a href='{{ root }}x'>",
                     {"index.pitch": "Hi <b>x</b>"}, {"root": "../"})
        self.assertEqual(out, "<p>Hi <b>x</b></p><a href='../x'>")

    def test_attr_escapes(self):
        out = render('<img alt="{{ attr:index.alt }}">', {"index.alt": 'a "b" <c>'}, {})
        self.assertEqual(out, '<img alt="a &quot;b&quot; &lt;c&gt;">')

    def test_unknown_placeholder_raises(self):
        with self.assertRaisesRegex(RenderError, "index.nope"):
            render("{{ index.nope }}", {}, {})

    def test_value_with_braces_raises(self):
        with self.assertRaises(RenderError):
            render("{{ index.x }}", {"index.x": "{{ x }}"}, {})


class StringsTest(unittest.TestCase):
    def test_hash_ignores_order(self):
        self.assertEqual(table_hash({"a": "1", "b": "2"}), table_hash({"b": "2", "a": "1"}))
        self.assertNotEqual(table_hash({"a": "1", "b": "2"}), table_hash({"a": "1", "b": "3"}))

    def test_texts_flattens_tables_only(self):
        data = {"language": "es", "common": {"x": "1"}, "index": {"y": "2"},
                "privacy": {}, "source-hash": {"common": "h"}}
        self.assertEqual(texts(data), {"common.x": "1", "index.y": "2"})


class PagesTest(unittest.TestCase):
    def setUp(self):
        self.out = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.out)

    def render_en(self, page):
        en = load("en")
        return render(read(os.path.join(landing.SITE, page)), texts(en),
                      page_values("en", page, en, [EN], self.out))

    def test_english_index_renders(self):
        html = self.render_en("index.html")
        self.assertIn('<html lang="en">', html)
        self.assertNotIn("ai-notice", html)
        want = master_assets()
        if want is not None:
            self.assertEqual(re.findall(r'data-asset="([^"]*)"', html), want)

    def test_english_privacy_renders(self):
        html = self.render_en("privacy.html")
        self.assertIn('<html lang="en">', html)
        self.assertNotIn("ai-notice", html)

    def test_translated_values(self):
        data = {"label": "L", "notice": "N", "privacy-notice": "P", "english-link": "E"}
        v = page_values("es", "privacy.html", data, [EN, ES], self.out)
        self.assertEqual(v["root"], "../")
        self.assertEqual(v["guide"], "../guide/es/")
        self.assertEqual(v["lang"], "es")
        self.assertEqual(v["shot"], "../images/main-screen.png")
        self.assertIn("P", v["notice"])
        self.assertNotIn("N ", v["notice"])
        self.assertIn('href="../privacy.html?lang=en"', v["notice"])
        self.assertIn('href="../privacy.html?lang=en"', v["menu"])
        self.assertRegex(v["menu"], r'<a href="../es/privacy.html"[^>]*aria-current="true"')
        self.assertIn('hreflang="x-default" href="https://sanchezfauste.com/fauste-player/privacy.html"',
                      v["alternates"])
        os.makedirs(os.path.join(self.out, "images", "es"))
        open(os.path.join(self.out, "images", "es", "main-screen.png"), "w").close()
        v = page_values("es", "privacy.html", data, [EN, ES], self.out)
        self.assertEqual(v["shot"], "../images/es/main-screen.png")

    def test_snap_badge_in_the_page_language(self):
        data = {"label": "L", "notice": "N", "privacy-notice": "P", "english-link": "E"}
        badge = "https://snapcraft.io/static/images/badges/{}/snap-store-white.svg"
        for lang, shown in (("es", "es"), ("de", "de"), ("ca", "en"), ("eu", "en")):
            v = page_values(lang, "index.html", data, [EN, ES], self.out)
            self.assertEqual(v["snap-badge"], badge.format(shown), lang)
        v = page_values("en", "index.html", load("en"), [EN, ES], self.out)
        self.assertEqual(v["snap-badge"], badge.format("en"))

    def test_microsoft_store_badge_in_the_page_language(self):
        data = {"label": "L", "notice": "N", "privacy-notice": "P", "english-link": "E"}
        badge = "https://get.microsoft.com/images/{}%20light.svg"
        for lang, shown in (("es", "es"), ("ca", "ca"), ("pt", "pt-pt"), ("eu", "en-us")):
            v = page_values(lang, "index.html", data, [EN, ES], self.out)
            self.assertEqual(v["msstore-badge"], badge.format(shown), lang)
        v = page_values("en", "index.html", load("en"), [EN, ES], self.out)
        self.assertEqual(v["msstore-badge"], badge.format("en-us"))

    def test_english_values(self):
        v = page_values("en", "index.html", load("en"), [EN, ES], self.out)
        self.assertEqual((v["root"], v["guide"], v["notice"]), ("", "guide/", ""))
        self.assertIn('href="?lang=en"', v["menu"])
        self.assertIn('href="es/"', v["menu"])

    def test_build_writes_each_language(self):
        locales = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, locales)
        shutil.copy(os.path.join(landing.LOCALES, "en.toml"), locales)
        en = read(os.path.join(locales, "en.toml"))
        body = en[en.index("[common]"):]
        with open(os.path.join(locales, "xx.toml"), "w", encoding="utf-8") as fh:
            fh.write('language = "xx"\nname = "Xx"\nlabel = "L"\nnotice = "N"\n'
                     'privacy-notice = "P"\nenglish-link = "E"\n' + body)
        build(self.out, locales=locales)
        for page in ("index.html", "privacy.html", "xx/index.html", "xx/privacy.html"):
            self.assertTrue(os.path.isfile(os.path.join(self.out, page)), page)
        html = read(os.path.join(self.out, "xx", "privacy.html"))
        self.assertIn('<html lang="xx">', html)
        self.assertIn('class="ai-notice"', html)


def toml_value(v):
    return json.dumps(v, ensure_ascii=False)


def write_translation(locales, code, en, meta=None, change=None):
    """A translation of `en` (the parsed English file) into `code`, with its
    hashes; `change` edits its tables before writing."""
    tables = {t: dict(en[t]) for t in TABLES}
    if change:
        change(tables)
    meta = dict({"language": code, "name": "Xx", "label": "L", "notice": "N",
                 "privacy-notice": "P", "english-link": "E"}, **(meta or {}))
    lines = [f"{k} = {toml_value(v)}" for k, v in meta.items()]
    lines.append("[source-hash]")
    lines += [f"{t} = {toml_value(table_hash(en[t]))}" for t in TABLES]
    for t in TABLES:
        lines.append(f"[{t}]")
        lines += [f"{toml_value(k)} = {toml_value(v)}" for k, v in tables[t].items()]
    with open(os.path.join(locales, f"{code}.toml"), "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")


class CheckTest(unittest.TestCase):
    def setUp(self):
        self.locales = tempfile.mkdtemp()
        self.i18n = tempfile.mkdtemp()
        self.out = tempfile.mkdtemp()
        for d in (self.locales, self.i18n, self.out):
            self.addCleanup(shutil.rmtree, d)
        shutil.copy(os.path.join(landing.LOCALES, "en.toml"), self.locales)
        self.en = load("en", self.locales)
        os.makedirs(os.path.join(self.i18n, "xx"))
        with open(os.path.join(self.i18n, "xx", "translation.toml"), "w") as fh:
            fh.write('name = "Xx"\nlabel = "L"\n')

    def problems(self, site=None):
        return check(site, self.locales, self.i18n)

    def test_valid_translation_passes(self):
        write_translation(self.locales, "xx", self.en)
        self.assertEqual(self.problems(), [])

    def test_language_sets_differ(self):
        write_translation(self.locales, "xx", self.en)
        os.makedirs(os.path.join(self.i18n, "yy"))
        got = self.problems()
        self.assertEqual(len(got), 1, got)
        self.assertTrue(got[0].startswith("site/locales: languages"), got)

    def test_stale_hash(self):
        write_translation(self.locales, "xx", self.en)
        en = read(os.path.join(self.locales, "en.toml"))
        with open(os.path.join(self.locales, "en.toml"), "w", encoding="utf-8") as fh:
            fh.write(en.replace('heading = "Privacy policy"', 'heading = "Privacy"'))
        new = table_hash(load("en", self.locales)["privacy"])
        got = self.problems()
        self.assertEqual(len(got), 1, got)
        self.assertIn("[privacy] was translated from older English text", got[0])
        self.assertIn(new, got[0])

    def test_key_sets(self):
        def change(t):
            del t["index"]["pitch"]
            t["index"]["extra"] = "x"
        write_translation(self.locales, "xx", self.en, change=change)
        got = "\n".join(self.problems())
        self.assertIn("[index] missing pitch", got)
        self.assertIn("[index] has keys English lacks: extra", got)

    def test_name_mismatch(self):
        write_translation(self.locales, "xx", self.en, meta={"name": "Other"})
        self.assertIn("site/locales/xx.toml: name differs from docs/i18n/xx/translation.toml",
                      self.problems())

    def test_missing_metadata(self):
        write_translation(self.locales, "xx", self.en, meta={"notice": ""})
        self.assertIn("site/locales/xx.toml: missing notice", self.problems())

    def test_placeholder_in_value(self):
        def change(t):
            t["index"]["pitch"] = "see {{ root }}"
        write_translation(self.locales, "xx", self.en, change=change)
        self.assertIn("site/locales/xx.toml: index.pitch contains a placeholder", self.problems())

    def test_built_site(self):
        write_translation(self.locales, "xx", self.en)
        build(self.out, locales=self.locales)
        self.assertEqual(self.problems(self.out), [])
        page = os.path.join(self.out, "xx", "privacy.html")
        html = read(page)
        with open(page, "w", encoding="utf-8") as fh:
            fh.write(re.sub(r'<div class="ai-notice".*?</div>', "", html, flags=re.S))
        self.assertIn("xx/privacy.html: no AI-translation notice", self.problems(self.out))


if __name__ == "__main__":
    unittest.main()
