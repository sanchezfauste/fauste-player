"""Tests of scripts/site/landing.py (python3 -m unittest discover -s
scripts/site/tests -p 'test_*.py')."""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from landing import RenderError, render, table_hash, texts  # noqa: E402


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


if __name__ == "__main__":
    unittest.main()
