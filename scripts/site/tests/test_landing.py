"""Tests of scripts/site/landing.py (python3 -m unittest discover -s
scripts/site/tests -p 'test_*.py')."""

import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

import landing  # noqa: E402
from landing import (RenderError, build, load, page_values, render,  # noqa: E402
                     table_hash, texts)

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


if __name__ == "__main__":
    unittest.main()
