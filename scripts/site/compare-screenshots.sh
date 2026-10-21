#!/usr/bin/env bash
# Compares freshly taken screenshots with the committed ones and reports
# which really changed:
#
#   scripts/site/compare-screenshots.sh [--fuzz <percent>] [--threshold <pixels>]
#                                       [--apply] [--prune] <new-dir> [<ref-dir>]
#
# <new-dir> is the output of `scripts/site/screenshots.sh --out <new-dir>`
# (main-screen.png and guide/*.png); <ref-dir> defaults to docs/images.
# Each PNG is compared with ImageMagick `compare -metric AE -fuzz <percent>`:
# pixels whose colours differ by less than the fuzz do not count, so font
# antialiasing noise is ignored. An image is "changed" when more than
# <pixels> pixels differ, or when its size differs.
#
#   --fuzz       colour tolerance in percent (default 5)
#   --threshold  differing pixels tolerated (default 50)
#   --apply      copy the changed and added images into <ref-dir>
#   --prune      with --apply, also delete the images missing from <new-dir>
#                (do not use it with a partial run such as screenshots.sh --only main)
#
# Output: one line per image, "changed|added|removed <path>" (paths relative
# to the folder), nothing when all is the same. When GITHUB_OUTPUT is set,
# "changed=true|false" is written to it. Exit status 0 even when images differ.
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
fuzz=5
threshold=50
apply=false
prune=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --fuzz) fuzz=${2:?--fuzz needs a percentage}; shift 2 ;;
        --threshold) threshold=${2:?--threshold needs a pixel count}; shift 2 ;;
        --apply) apply=true; shift ;;
        --prune) prune=true; shift ;;
        -*) echo "usage: compare-screenshots.sh [--fuzz <pct>] [--threshold <px>] [--apply] [--prune] <new-dir> [<ref-dir>]" >&2; exit 2 ;;
        *) break ;;
    esac
done
[[ $# -ge 1 && $# -le 2 ]] || { echo "usage: compare-screenshots.sh [options] <new-dir> [<ref-dir>]" >&2; exit 2; }
new=${1%/}
ref=${2:-$root/docs/images}
ref=${ref%/}
[[ -d $new ]] || { echo "compare-screenshots.sh: $new is not a folder" >&2; exit 2; }
command -v compare >/dev/null || { echo "compare-screenshots.sh: ImageMagick (compare) is needed" >&2; exit 1; }

any=false
while IFS= read -r rel; do
    if [[ ! -f $ref/$rel ]]; then
        echo "added $rel"
        any=true
        if $apply; then mkdir -p "$(dirname "$ref/$rel")"; cp "$new/$rel" "$ref/$rel"; fi
        continue
    fi
    # compare prints the count of differing pixels on stderr; a size
    # mismatch makes it fail without a count, which also means "changed".
    diff=$(compare -metric AE -fuzz "${fuzz}%" "$new/$rel" "$ref/$rel" null: 2>&1 || true)
    diff=${diff%% *}
    if [[ ! $diff =~ ^[0-9]+$ ]] || (( diff > threshold )); then
        echo "changed $rel"
        any=true
        if $apply; then cp "$new/$rel" "$ref/$rel"; fi
    fi
done < <(cd "$new" && find . -name '*.png' -type f | sed 's|^\./||' | sort)

while IFS= read -r rel; do
    [[ -f $new/$rel ]] && continue
    echo "removed $rel"
    any=true
    if $apply && $prune; then rm -f "$ref/$rel"; fi
done < <(cd "$ref" && find . -name '*.png' -type f | sed 's|^\./||' | sort)

if [[ -n ${GITHUB_OUTPUT:-} ]]; then echo "changed=$any" >> "$GITHUB_OUTPUT"; fi
