#!/usr/bin/env python3
"""Builds and checks the user guide in every language.

The English guide in docs/user is the only source. Each translation lives in
docs/i18n/<lang>/ with the same file names and SUMMARY entries, plus
translation.toml, which records the English commit it was translated from and
the strings of the AI-translation notice (see docs/technical/release-process.md).

    guide.py build <mdbook> <out-dir>   English book in <out-dir>, each
                                        translation in <out-dir>/<lang>
    guide.py check [<site-dir>]         translations match their source;
                                        with a built site, every translated
                                        page carries the notice
    guide.py images <site-dir> <shots>  screenshots taken in each language
                                        (<shots>/<lang>/guide/*.png) into the
                                        site; each translated book links its
                                        own, else the English ones
    guide.py changed [<lang>...]        English pages changed since each
                                        translation's source commit

Needs Python 3.11 or later (tomllib).
"""

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib

ROOT = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))
DOCS = os.path.join(ROOT, "docs")
SOURCE = os.path.join(DOCS, "user")
I18N = os.path.join(DOCS, "i18n")
META = "translation.toml"
ASSETS = "book-assets"
DATA_JS = "languages.js"
NOTICE_CLASS = "ai-notice"

ENGLISH = {"code": "en", "name": "English", "label": "Language"}
REQUIRED = ("language", "name", "title", "label", "source-commit",
            "source-version", "notice", "english-link")

LINK = re.compile(r"^(\s*)(?:- )?\[[^\]]*\]\(([^)]+)\)\s*$")


def fail(msg):
    print(f"guide.py: {msg}", file=sys.stderr)
    sys.exit(1)


def languages():
    """Translation codes, sorted (the folders of docs/i18n)."""
    if not os.path.isdir(I18N):
        return []
    return sorted(d for d in os.listdir(I18N)
                  if os.path.isdir(os.path.join(I18N, d)))


def load_meta(lang):
    path = os.path.join(I18N, lang, META)
    with open(path, "rb") as fh:
        return tomllib.load(fh)


def summary_entries(text):
    """(indent, target) of every link in a SUMMARY.md, in order."""
    out = []
    for line in text.splitlines():
        m = LINK.match(line)
        if m:
            out.append((len(m.group(1)), m.group(2)))
    return out


def pages(summary_text):
    """The HTML pages a book built from this SUMMARY has."""
    out = []
    for _, target in summary_entries(summary_text):
        name = target.split("#")[0]
        out.append("index.html" if name == "README.md" else name[:-3] + ".html")
    return out


def read(path):
    with open(path, encoding="utf-8") as fh:
        return fh.read()


def notice_text(meta):
    return meta["notice"].format(version=meta["source-version"],
                                 commit=meta["source-commit"][:7])


# --- build -----------------------------------------------------------------

def build(mdbook, out):
    langs = languages()
    menu = [dict(ENGLISH, pages=pages(read(os.path.join(SOURCE, "SUMMARY.md"))))]
    metas = {}
    for lang in langs:
        meta = load_meta(lang)
        metas[lang] = meta
        menu.append({"code": lang, "name": meta["name"],
                     "pages": pages(read(os.path.join(I18N, lang, "SUMMARY.md")))})

    with tempfile.TemporaryDirectory(prefix="fauste-guide-") as stage:
        # A staging copy of docs/ keeps the generated languages.js and the
        # metadata files out of the source tree and out of the books.
        shutil.copy(os.path.join(DOCS, "book.toml"), stage)
        shutil.copytree(os.path.join(DOCS, ASSETS), os.path.join(stage, ASSETS))
        shutil.copytree(SOURCE, os.path.join(stage, "user"))
        if langs:
            shutil.copytree(I18N, os.path.join(stage, "i18n"),
                            ignore=shutil.ignore_patterns(META))

        for lang in ["en"] + langs:
            data = {"current": lang, "languages": [
                {"code": m["code"], "name": m["name"], "pages": m["pages"]} for m in menu]}
            env = dict(os.environ)
            if lang == "en":
                data["label"] = ENGLISH["label"]
                data["notice"] = None
                dest = out
            else:
                meta = metas[lang]
                data["label"] = meta["label"]
                data["notice"] = {"text": notice_text(meta), "link": meta["english-link"]}
                dest = os.path.join(out, lang)
                env["MDBOOK_BOOK__SRC"] = f"i18n/{lang}"
                env["MDBOOK_BOOK__LANGUAGE"] = lang
                env["MDBOOK_BOOK__TITLE"] = meta["title"]
            with open(os.path.join(stage, ASSETS, DATA_JS), "w", encoding="utf-8") as fh:
                fh.write("// Generated by scripts/site/guide.py.\n")
                fh.write("window.faustePlayerGuide = ")
                json.dump(data, fh, ensure_ascii=False)
                fh.write(";\n")
            subprocess.run([mdbook, "build", stage, "-d", dest], env=env, check=True)


# --- check -----------------------------------------------------------------

def git(*args):
    try:
        return subprocess.run(["git", "-C", ROOT, *args], capture_output=True,
                              text=True, check=True).stdout
    except (OSError, subprocess.CalledProcessError):
        return None


def source_at(commit):
    """{file name: text} of docs/user at a commit, or None without history."""
    names = git("ls-tree", "--name-only", f"{commit}:docs/user")
    if names is None:
        return None
    out = {}
    for name in names.split():
        if name.endswith(".md"):
            out[name] = git("show", f"{commit}:docs/user/{name}") or ""
    return out


def shallow():
    """True when the clone is shallow or git cannot say."""
    return git("rev-parse", "--is-shallow-repository") != "false\n"


def hyphen_problems(where, name, text):
    """Lines of a translated page that end in a letter and a hyphen.

    A line-end hyphen in a compound renders as "word- word" (the line break
    becomes a space), so compounds must not be split across lines.
    """
    out = []
    fence = False
    for i, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith(("```", "~~~")):
            fence = not fence
        elif not fence and re.search(r"[^\W\d_]-$", line):
            out.append(f"{where}/{name}:{i}: line ends in a hyphen "
                       "(it renders with a space; keep the compound on one line)")
    return out


def check(site):
    problems = []
    langs = languages()
    for lang in langs:
        where = f"docs/i18n/{lang}"
        try:
            meta = load_meta(lang)
        except (OSError, tomllib.TOMLDecodeError) as e:
            problems.append(f"{where}/{META}: {e}")
            continue
        missing = [k for k in REQUIRED if not str(meta.get(k, "")).strip()]
        if missing:
            problems.append(f"{where}/{META}: missing {', '.join(missing)}")
            continue
        if meta["language"] != lang:
            problems.append(f"{where}/{META}: language is {meta['language']!r}, not {lang!r}")
        commit = meta["source-commit"]
        if not re.fullmatch(r"[0-9a-f]{40}", commit):
            problems.append(f"{where}/{META}: source-commit must be a full commit hash")
            continue
        try:
            notice_text(meta)
        except (KeyError, IndexError, ValueError) as e:
            problems.append(f"{where}/{META}: notice has a bad placeholder ({e})")

        # The translation mirrors the English guide at its source commit;
        # English pages added later fall back to the start page in the
        # language menu until the next regeneration.
        source = source_at(commit)
        if source is None and not shallow() and git("cat-file", "-e", f"{commit}^{{commit}}") is None:
            problems.append(f"{where}/{META}: source-commit {commit[:7]} does not exist")
            continue
        if source is None:
            print(f"guide.py: {where}: no git history for {commit[:7]}; "
                  "comparing with the current docs/user", file=sys.stderr)
            source = {n: read(os.path.join(SOURCE, n))
                      for n in os.listdir(SOURCE) if n.endswith(".md")}
        elif git("cat-file", "-e", f"{commit}^{{commit}}") is None:
            problems.append(f"{where}/{META}: source-commit {commit[:7]} does not exist")
        elif git("merge-base", "--is-ancestor", commit, "HEAD") is None:
            problems.append(f"{where}/{META}: source-commit {commit[:7]} is not an ancestor of HEAD")

        files = {n for n in os.listdir(os.path.join(I18N, lang)) if n != META}
        for n in sorted(set(source) - files):
            problems.append(f"{where}: missing {n}")
        for n in sorted(files - set(source)):
            problems.append(f"{where}: {n} is not in the English guide at {commit[:7]}")

        for n in sorted(files):
            if n.endswith(".md"):
                problems.extend(hyphen_problems(where, n, read(os.path.join(I18N, lang, n))))

        summary = os.path.join(I18N, lang, "SUMMARY.md")
        if "SUMMARY.md" in source and os.path.isfile(summary):
            want = summary_entries(source["SUMMARY.md"])
            got = summary_entries(read(summary))
            if want != got:
                problems.append(f"{where}/SUMMARY.md: entries differ from the English "
                                f"SUMMARY at {commit[:7]} (same targets, order and nesting)")

        readme = os.path.join(I18N, lang, "README.md")
        if os.path.isfile(readme):
            text = read(readme)
            if f'class="{NOTICE_CLASS}"' not in text:
                problems.append(f"{where}/README.md: no AI-translation notice "
                                f'(<div class="{NOTICE_CLASS}">)')
            elif meta["source-version"] not in text or commit[:7] not in text:
                problems.append(f"{where}/README.md: the notice does not name "
                                f"{meta['source-version']} and {commit[:7]}")

        if site:
            book = os.path.join(site, "guide", lang)
            built = [os.path.join(d, f) for d, _, fs in os.walk(book)
                     for f in fs if f.endswith(".html")]
            if not built:
                problems.append(f"guide/{lang}: not built")
            for page in built:
                html = read(page)
                rel = os.path.relpath(page, site)
                if f'<html lang="{lang}"' not in html:
                    problems.append(f"{rel}: not marked lang={lang}")
                if os.path.basename(page) == "toc.html":
                    continue  # the sidebar's fallback for browsers without JavaScript
                if not re.search(r'src="[^"]*languages-[0-9a-f]+\.js"', html) or \
                   not re.search(r'src="[^"]*language-menu-[0-9a-f]+\.js"', html):
                    problems.append(f"{rel}: no language menu and notice script")

    if site:
        index = os.path.join(site, "guide", "index.html")
        if not os.path.isfile(index) or "language-menu-" not in read(index):
            problems.append("guide/index.html: no language menu script")

    for p in problems:
        print(p)
    if problems:
        print(f"{len(problems)} problem(s) in the translations")
        sys.exit(1)
    print(f"{len(langs)} translation(s) checked: {', '.join(langs) or 'none'}")


# --- images ----------------------------------------------------------------

def images(site, shots):
    """Copies the per-language screenshots into the site and points each
    translated book at its own.

    <shots>/<lang>/ holds main-screen.png and guide/*.png (scripts/site/
    localized-screenshots.sh). English ones replace the committed images in
    <site>/images; the others go to <site>/images/<lang>/. A translation
    links ../../images/guide/<name>; it is rewritten to its own copy only
    when that copy exists, so a missing screenshot keeps the English one.
    """
    if not os.path.isdir(shots):
        fail(f"{shots}: no such folder")
    pictures = os.path.join(site, "images")
    rewritten = 0
    for lang in sorted(os.listdir(shots)):
        src = os.path.join(shots, lang)
        if not os.path.isdir(src):
            continue
        if lang == "en":
            shutil.copytree(src, pictures, dirs_exist_ok=True)
            continue
        if lang not in languages():
            fail(f"{src}: {lang} is not a translation in docs/i18n")
        shutil.copytree(src, os.path.join(pictures, lang), dirs_exist_ok=True)
        book = os.path.join(site, "guide", lang)
        for d, _, files in os.walk(book):
            for f in files:
                if not f.endswith(".html"):
                    continue
                path = os.path.join(d, f)
                html = read(path)

                def own(m):
                    name = m.group(2)
                    if os.path.isfile(os.path.join(pictures, lang, "guide", name)):
                        return f"{m.group(1)}images/{lang}/guide/{name}"
                    return m.group(0)

                new = re.sub(r'((?:\.\./)+)images/guide/([^"\'#?)\s]+)', own, html)
                if new != html:
                    rewritten += 1
                    with open(path, "w", encoding="utf-8") as fh:
                        fh.write(new)
    print(f"screenshots in {len(os.listdir(shots))} language(s); "
          f"{rewritten} translated page(s) use their own")


# --- changed ---------------------------------------------------------------

def changed(langs):
    for lang in langs or languages():
        meta = load_meta(lang)
        commit = meta["source-commit"]
        out = git("diff", "--name-status", commit, "HEAD", "--", "docs/user")
        if out is None:
            fail(f"{lang}: cannot diff from {commit[:7]} (needs the git history)")
        print(f"{lang}: translated from {meta['source-version']} ({commit[:7]})")
        lines = [l for l in out.splitlines() if l.strip()]
        for line in lines:
            print("  " + line)
        if not lines:
            print("  up to date")


def main(argv):
    if len(argv) >= 1 and argv[0] == "build" and len(argv) == 3:
        build(argv[1], argv[2])
    elif len(argv) >= 1 and argv[0] == "check" and len(argv) <= 2:
        check(argv[1] if len(argv) == 2 else None)
    elif len(argv) == 3 and argv[0] == "images":
        images(argv[1], argv[2])
    elif len(argv) >= 1 and argv[0] == "changed":
        changed(argv[1:])
    else:
        print(__doc__, file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main(sys.argv[1:])
