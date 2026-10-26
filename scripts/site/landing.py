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


# The languages snapcraft.io draws its "Get it from the Snap Store" badge in;
# the others show the English one.
SNAP_BADGES = {"de", "en", "es", "fr", "it", "pl", "pt"}
SNAP_BADGE = "https://snapcraft.io/static/images/badges/{}/snap-store-white.svg"

# The Microsoft Store badge's name for each language it is drawn in; the others
# show the US English one.
MSSTORE_BADGES = {"ca": "ca", "de": "de", "es": "es", "fr": "fr", "gl": "gl", "it": "it",
                  "nl": "nl", "pl": "pl", "pt": "pt-pt"}
MSSTORE_BADGE = "https://get.microsoft.com/images/{}%20light.svg"


GLOBE = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="1.1em" '
         'height="1.1em" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">'
         '<circle cx="12" cy="12" r="10"/><path d="M2 12h20"/>'
         '<path d="M12 2a15 15 0 0 1 0 20a15 15 0 0 1 0-20z"/></svg>')


def codes(locales=LOCALES):
    """The translation codes: the strings files other than en.toml."""
    return sorted(f[:-5] for f in os.listdir(locales)
                  if f.endswith(".toml") and f != "en.toml")


def folder(code):
    return "" if code == "en" else f"{code}/"


def page_values(lang, page, data, langs, out):
    """The build values of one page in one language. `langs` lists
    {"code", "name"}, English first; `out` is the site being built, where a
    localized screenshot may already be."""
    esc = html.escape
    root = "" if lang == "en" else "../"
    target = "" if page == "index.html" else page
    label = esc(data["label"], quote=True)

    items = []
    for l in langs:
        href = root + folder(l["code"]) + target + ("?lang=en" if l["code"] == "en" else "")
        current = ' aria-current="true"' if l["code"] == lang else ""
        items.append(f'<li><a href="{href}" lang="{l["code"]}" hreflang="{l["code"]}"{current}>'
                     f'{esc(l["name"])}</a></li>')
    menu = (f'<details class="language-menu"><summary title="{label}" aria-label="{label}">'
            f'{GLOBE}</summary><ul>{"".join(items)}</ul></details>')

    alternates = [f'<link rel="alternate" hreflang="{l["code"]}" '
                  f'href="{BASE_URL}{folder(l["code"])}{target}">' for l in langs]
    alternates.append(f'<link rel="alternate" hreflang="x-default" href="{BASE_URL}{target}">')

    notice = ""
    if lang != "en":
        text = data["notice" if page == "index.html" else "privacy-notice"]
        notice = (f'<div class="ai-notice" role="note">{esc(text)} '
                  f'<a href="{root}{target}?lang=en" hreflang="en">{esc(data["english-link"])}</a></div>')

    localized = os.path.isfile(os.path.join(out, "images", lang, "main-screen.png"))
    return {
        "root": root,
        "guide": root + "guide/" + folder(lang),
        "lang": lang,
        "shot": root + "images/" + (f"{lang}/" if lang != "en" and localized else "") + "main-screen.png",
        "menu": menu,
        "alternates": "\n".join(alternates),
        "notice": notice,
        "other-languages": ", ".join(l["name"] for l in langs if l["code"] != lang),
        "snap-badge": SNAP_BADGE.format(lang if lang in SNAP_BADGES else "en"),
        "msstore-badge": MSSTORE_BADGE.format(MSSTORE_BADGES.get(lang, "en-us")),
    }


def build(out, locales=LOCALES, site=SITE):
    """Renders every page in English into <out> and in each translation into
    <out>/<code>."""
    files = {code: load(code, locales) for code in ["en"] + codes(locales)}
    langs = [{"code": code, "name": data["name"]} for code, data in files.items()]
    templates = {page: open(os.path.join(site, page), encoding="utf-8").read() for page in PAGES}
    for code, data in files.items():
        dest = os.path.join(out, folder(code))
        os.makedirs(dest, exist_ok=True)
        for page, template in templates.items():
            try:
                rendered = render(template, texts(data),
                                  page_values(code, page, data, langs, out))
            except RenderError as e:
                raise RenderError(f"{code}/{page}: {e}") from None
            with open(os.path.join(dest, page), "w", encoding="utf-8") as fh:
                fh.write(rendered)


def check(site, locales=LOCALES, i18n=I18N):
    """Problems with the translations, as lines; with a built site, also
    with their pages. Empty when everything is fine."""
    problems = []
    langs = codes(locales)
    guide = sorted(d for d in os.listdir(i18n) if os.path.isdir(os.path.join(i18n, d)))
    if langs != guide:
        problems.append(f"site/locales: languages {' '.join(langs) or '(none)'} differ from "
                        f"docs/i18n {' '.join(guide) or '(none)'}")
    try:
        en = load("en", locales)
    except (OSError, tomllib.TOMLDecodeError) as e:
        return problems + [f"site/locales/en.toml: {e}"]
    hashes = {t: table_hash(en.get(t, {})) for t in TABLES}

    for code in langs:
        where = f"site/locales/{code}.toml"
        try:
            data = load(code, locales)
        except (OSError, tomllib.TOMLDecodeError) as e:
            problems.append(f"{where}: {e}")
            continue
        recorded = data.get("source-hash", {})
        missing = [k for k in META_KEYS if not str(data.get(k, "")).strip()]
        missing += [f"source-hash.{t}" for t in TABLES if not recorded.get(t)]
        if missing:
            problems.append(f"{where}: missing {', '.join(missing)}")
        if data.get("language", code) != code:
            problems.append(f"{where}: language is {data['language']!r}, not {code!r}")
        try:
            with open(os.path.join(i18n, code, "translation.toml"), "rb") as fh:
                meta = tomllib.load(fh)
        except (OSError, tomllib.TOMLDecodeError):
            meta = None
        for key in ("name", "label"):
            if meta is not None and data.get(key) and data.get(key) != meta.get(key):
                problems.append(f"{where}: {key} differs from docs/i18n/{code}/translation.toml")
        for t in TABLES:
            want, got = set(en.get(t, {})), set(data.get(t, {}))
            if want - got:
                problems.append(f"{where}: [{t}] missing {', '.join(sorted(want - got))}")
            if got - want:
                problems.append(f"{where}: [{t}] has keys English lacks: {', '.join(sorted(got - want))}")
            if recorded.get(t) and recorded[t] != hashes[t]:
                problems.append(f"{where}: [{t}] was translated from older English text "
                                f"(source-hash should be {hashes[t]})")
        for name, value in texts(data).items():
            if "{{" in value:
                problems.append(f"{where}: {name} contains a placeholder")

        if site:
            for page in PAGES:
                rel = f"{code}/{page}"
                path = os.path.join(site, code, page)
                if not os.path.isfile(path):
                    problems.append(f"{rel}: not built")
                    continue
                with open(path, encoding="utf-8") as fh:
                    built = fh.read()
                if f'<html lang="{code}"' not in built:
                    problems.append(f"{rel}: not marked lang={code}")
                if 'class="ai-notice"' not in built:
                    problems.append(f"{rel}: no AI-translation notice")
                if 'class="language-menu"' not in built:
                    problems.append(f"{rel}: no language menu")
    return problems


def main(argv):
    if argv == ["hash"]:
        en = load("en")
        print("[source-hash]")
        for t in TABLES:
            print(f'{t} = "{table_hash(en.get(t, {}))}"')
    elif len(argv) == 2 and argv[0] == "build":
        try:
            build(argv[1])
        except (OSError, tomllib.TOMLDecodeError, KeyError, RenderError) as e:
            print(f"landing.py: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        print(__doc__, file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main(sys.argv[1:])
