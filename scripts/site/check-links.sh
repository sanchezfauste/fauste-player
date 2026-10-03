#!/bin/sh
# Usage: check-links.sh <out-dir>
# Fails (exit 1) when an <img src> or a relative href in the built site's
# HTML pages does not resolve to a file inside <out-dir>.
set -eu

[ $# -eq 1 ] || { echo "usage: $0 <out-dir>" >&2; exit 2; }
exec python3 - "$1" <<'PY'
import os, sys, re
from html.parser import HTMLParser
from urllib.parse import urlsplit, unquote

out = os.path.realpath(sys.argv[1])
bad = 0

class P(HTMLParser):
    def __init__(self, page):
        super().__init__()
        self.page = page
        self.ids = set()
        self.refs = []
    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if "id" in a:
            self.ids.add(a["id"])
        if tag == "a" and a.get("name"):
            self.ids.add(a["name"])
        if tag in ("img", "script", "source") and a.get("src"):
            self.refs.append(a["src"])
        if tag in ("a", "link") and a.get("href"):
            self.refs.append(a["href"])
        if tag == "a" and "data-asset" in a and a.get("href", "").endswith("/releases/latest"):
            self.refs.append("!placeholder:" + a["data-asset"])

pages = {}
for d, _, files in os.walk(out):
    for f in files:
        if f.endswith(".html"):
            p = os.path.join(d, f)
            parser = P(p)
            with open(p, encoding="utf-8") as fh:
                parser.feed(fh.read())
            pages[p] = parser

for page, parser in pages.items():
    rel = os.path.relpath(page, out)
    for ref in parser.refs:
        if ref.startswith("!placeholder:"):
            print(f"{rel}: download link not filled in ({ref[13:]})")
            bad += 1
            continue
        u = urlsplit(ref)
        if u.scheme or u.netloc or ref.startswith(("mailto:", "javascript:")):
            continue
        path = unquote(u.path)
        if path == "":
            target = page
        elif path.startswith("/"):
            target = os.path.join(out, path.lstrip("/"))
        else:
            target = os.path.normpath(os.path.join(os.path.dirname(page), path))
        if os.path.isdir(target):
            target = os.path.join(target, "index.html")
        real = os.path.realpath(target)
        if not (real == out or real.startswith(out + os.sep)) or not os.path.isfile(real):
            print(f"{rel}: dead link {ref}")
            bad += 1
            continue
        if u.fragment and real in pages and u.fragment not in pages[real].ids:
            if not real.endswith("print.html"):
                print(f"{rel}: missing anchor {ref}")
                bad += 1

if bad:
    print(f"{bad} dead link(s)")
    sys.exit(1)
print(f"{len(pages)} pages checked, all links resolve")
PY
