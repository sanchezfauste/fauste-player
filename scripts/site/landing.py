#!/usr/bin/env python3
"""Renders and checks the landing page and the privacy policy in every
language.

site/index.html and site/privacy.html are templates. Their texts live in
site/locales/<lang>.toml: en.toml is the only source, and every other file
is a translation into a language of the user guide (docs/i18n). Unlike the
guide, the translations never lag behind English: each one records the hash
of the English texts it was made from (see docs/technical/release-process.md).

    landing.py build <out-dir>   English pages in <out-dir>, each
                                 translation in <out-dir>/<lang>
    landing.py hash              the [source-hash] table of the current
                                 English texts, to paste into a translation

The check runs as part of scripts/site/check-translations.sh.

Needs Python 3.11 or later (tomllib).
"""

import hashlib
import html
import json
import os
import re
import sys
import tomllib

ROOT = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))
SITE = os.path.join(ROOT, "site")
LOCALES = os.path.join(SITE, "locales")
I18N = os.path.join(ROOT, "docs", "i18n")
TABLES = ("common", "index", "privacy")
META_KEYS = ("language", "name", "label", "notice", "privacy-notice", "english-link")
PAGES = {"index.html": "index", "privacy.html": "privacy"}
BASE_URL = os.environ.get("FAUSTE_SITE_URL", "https://sanchezfauste.com/fauste-player/")

PLACEHOLDER = re.compile(r"\{\{\s*(attr:)?([a-z0-9_.-]+)\s*\}\}")


class RenderError(Exception):
    pass


def load(lang, locales=LOCALES):
    with open(os.path.join(locales, f"{lang}.toml"), "rb") as fh:
        return tomllib.load(fh)


def texts(data):
    """The texts of a strings file as "table.key" -> value."""
    return {f"{table}.{key}": value
            for table in TABLES for key, value in data.get(table, {}).items()}


def table_hash(table):
    """The hash of one table's texts; comments and formatting do not count."""
    canonical = json.dumps(table, sort_keys=True, ensure_ascii=False, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def render(template, texts, values):
    """Fills {{ table.key }} with a text and {{ name }} with a build value;
    {{ attr:… }} escapes it for an attribute. Unknown names fail."""
    def fill(m):
        name = m.group(2)
        source = texts if "." in name else values
        if name not in source:
            raise RenderError(f"unknown placeholder {name}")
        value = source[name]
        return html.escape(value, quote=True) if m.group(1) else value

    out = PLACEHOLDER.sub(fill, template)
    if "{{" in out:
        raise RenderError("placeholder left in output")
    return out


def main(argv):
    print(__doc__, file=sys.stderr)
    sys.exit(2)


if __name__ == "__main__":
    main(sys.argv[1:])
