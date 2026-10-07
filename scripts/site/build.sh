#!/bin/sh
# Builds the website into <out-dir> (default target/site): the landing page
# (site/) with its download links filled from the latest release, the user
# guide under guide/ (each translation under guide/<lang>/) and the
# screenshots under images/.
#
# The release comes from `gh release view --json tagName,url,assets`, or from
# the JSON file named by FAUSTE_RELEASE_JSON (tests, offline builds). Without
# a release the page is still built, and its download links point at the
# releases page. The guide is built by scripts/site/guide.py (Python 3.11+).
set -eu

root=$(cd "$(dirname "$0")/../.." && pwd)
out=${1:-$root/target/site}

mdbook=$("$root/scripts/site/mdbook.sh")

# The output folder is replaced. Inside the repository only target/... and
# _site may be; outside it, only a missing or empty folder, or one this
# script built before (it holds the marker file).
marker=.fauste-site
parent=$(cd "$(dirname "$out")" 2>/dev/null && pwd) || {
    echo "build.sh: the parent of $out does not exist" >&2; exit 1; }
abs=${parent%/}/$(basename "$out")
case $abs in
  "$root"/target/?* | "$root"/_site) ;;
  "$root" | "$root"/*)
    echo "build.sh: refusing to replace $out (inside the repository, use target/... or _site)" >&2
    exit 1 ;;
  *)
    case $root/ in
      "$abs"/*) echo "build.sh: refusing to replace $out (a parent of the repository)" >&2; exit 1 ;;
    esac
    if [ -d "$abs" ] && [ ! -f "$abs/$marker" ] && [ -n "$(ls -A "$abs")" ]; then
        echo "build.sh: refusing to replace $out (not empty and not built by this script)" >&2
        exit 1
    fi ;;
esac
case $abs in
  */. | */..) echo "build.sh: refusing to replace $out" >&2; exit 1 ;;
esac
rm -rf "$abs"
mkdir -p "$abs"
out=$abs
: > "$out/$marker"

# The guide: English in guide/, each translation of docs/i18n in guide/<lang>/.
python3 "$root/scripts/site/guide.py" build "$mdbook" "$out/guide"
mkdir -p "$out/images"
cp -R "$root"/docs/images/. "$out/images/"

cp "$root"/site/* "$out/"
cp "$root/packaging/icons/fauste-player.svg" "$out/favicon.svg"

release=
if [ -n "${FAUSTE_RELEASE_JSON:-}" ]; then
    release=$(cat "$FAUSTE_RELEASE_JSON") || release=
elif command -v gh >/dev/null 2>&1; then
    release=$(gh release view --json tagName,url,assets 2>/dev/null) || release=
fi
if [ -z "$release" ]; then
    echo "build.sh: no release found; download links point at the releases page" >&2
fi

RELEASE_JSON=$release python3 - "$out/index.html" <<'PY'
import json, os, re, sys

path = sys.argv[1]
raw = os.environ.get("RELEASE_JSON", "")
try:
    release = json.loads(raw) if raw.strip() else None
except ValueError:
    print("build.sh: release JSON is not valid; ignoring it", file=sys.stderr)
    release = None

html = open(path, encoding="utf-8").read()

if release is None:
    html = re.sub(r'\sdata-asset="[^"]*"', "", html)
    html = re.sub(r"\sdata-row(?=>)", "", html)
    html = html.replace("<span data-version></span>", "").replace(
        '<span class="version" data-version></span>', '<span class="version"></span>')
else:
    assets = [(a["name"], a["url"]) for a in release.get("assets", [])
              if not a["name"].endswith(".sha256")]
    version = release.get("tagName", "")

    link = re.compile(r'<a data-asset="([^"]*)" href="[^"]*">(.*?)</a>', re.S)

    def fill_row(match):
        tag, body = match.group(1), match.group(2)
        found = [0]

        def fill_link(m):
            pattern = re.compile(m.group(1))
            for name, url in assets:
                if pattern.search(name):
                    found[0] += 1
                    return f'<a href="{url}">{m.group(2)}</a>'
            return None  # no such asset

        def sub(m):
            repl = fill_link(m)
            return repl if repl is not None else "\0MISSING\0"

        body = link.sub(sub, body)
        if not found[0]:
            return ""
        if tag == "tr":
            body = re.sub(r"<td>\0MISSING\0</td>", "<td>—</td>", body)
        body = body.replace("\0MISSING\0", "")
        return f"<{tag}>{body}</{tag}>"

    html = re.sub(r"<(li|tr) data-row>(.*?)</\1>", fill_row, html, flags=re.S)
    html = html.replace("<span data-version></span>", version)
    html = html.replace('<span class="version" data-version></span>',
                        f'<span class="version">{version}</span>')

open(path, "w", encoding="utf-8").write(html)
PY

echo "Site built in $out"
