#!/usr/bin/env bash
# Takes the README and guide screenshots in English and in every language of
# the guide (a folder of docs/i18n), for the published site:
#
#   scripts/site/localized-screenshots.sh <out-dir> [<lang>...]
#
# The images go to <out-dir>/<lang>/ (main-screen.png and guide/*.png), the
# layout FAUSTE_SHOTS_DIR expects in scripts/site/build.sh. Each guide
# language uses the interface locale of crates/fp-app/locales that starts
# with its code (pt -> pt-PT, es -> es-ES); a language without one is
# skipped, and its book keeps the English images. With no language given,
# English and all translations are done. The release binary is built once
# (screenshots.sh builds on its first run; later runs find it up to date).
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
[[ $# -ge 1 ]] || { echo "usage: localized-screenshots.sh <out-dir> [<lang>...]" >&2; exit 2; }
out=$1; shift
langs=("$@")
if [[ ${#langs[@]} -eq 0 ]]; then
    langs=(en)
    for d in "$root"/docs/i18n/*/; do langs+=("$(basename "$d")"); done
fi

for lang in "${langs[@]}"; do
    locale=
    for f in "$root"/crates/fp-app/locales/"$lang"-*/; do
        [[ -d $f ]] && { locale=$(basename "$f"); break; }
    done
    if [[ -z $locale ]]; then
        echo "localized-screenshots.sh: no interface locale for '$lang'; its guide keeps the English images" >&2
        continue
    fi
    echo "localized-screenshots.sh: $lang ($locale)" >&2
    "$root/scripts/site/screenshots.sh" --lang "$locale" --out "$out/$lang"
done
