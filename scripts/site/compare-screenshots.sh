#!/usr/bin/env bash
# Compares freshly taken screenshots with the committed ones and reports
# which really changed:
#
#   scripts/site/compare-screenshots.sh [--fuzz <percent>] [--threshold <pixels>]
#                                       [--masks <file>] [--apply] [--prune]
#                                       <new-dir> [<ref-dir>]
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
#   --masks      regions that change on every run (the clock, elapsed times,
#                meters, the cart sweep): lines "<image> <x>,<y>,<w>,<h>"
#                (default scripts/site/screenshot-masks.txt; "#" comments).
#                The rectangles are painted on both images before comparing.
#   --apply      copy the changed and added images into <ref-dir>
#   --prune      with --apply, also delete main-screen.png and guide/*.png when
#                missing from <new-dir> (never with a partial run such as
#                screenshots.sh --only main); other images are left alone
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
masks=$root/scripts/site/screenshot-masks.txt
while [[ $# -gt 0 ]]; do
    case $1 in
        --fuzz) fuzz=${2:?--fuzz needs a percentage}; shift 2 ;;
        --threshold) threshold=${2:?--threshold needs a pixel count}; shift 2 ;;
        --masks) masks=${2:?--masks needs a file}; shift 2 ;;
        --apply) apply=true; shift ;;
        --prune) prune=true; shift ;;
        -*) echo "usage: compare-screenshots.sh [--fuzz <pct>] [--threshold <px>] [--masks <file>] [--apply] [--prune] <new-dir> [<ref-dir>]" >&2; exit 2 ;;
        *) break ;;
    esac
done
[[ $# -ge 1 && $# -le 2 ]] || { echo "usage: compare-screenshots.sh [options] <new-dir> [<ref-dir>]" >&2; exit 2; }
new=${1%/}
ref=${2:-$root/docs/images}
ref=${ref%/}
[[ -d $new ]] || { echo "compare-screenshots.sh: $new is not a folder" >&2; exit 2; }
command -v compare >/dev/null && command -v identify >/dev/null || { echo "compare-screenshots.sh: ImageMagick (compare) is needed" >&2; exit 1; }

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
any=false
while IFS= read -r rel; do
    if [[ ! -f $ref/$rel ]]; then
        echo "added $rel"
        any=true
        if $apply; then mkdir -p "$(dirname "$ref/$rel")"; cp "$new/$rel" "$ref/$rel"; fi
        continue
    fi
    # Different sizes cannot be compared pixel by pixel: changed.
    if [[ $(identify -format '%wx%h' "$new/$rel") != "$(identify -format '%wx%h' "$ref/$rel")" ]]; then
        echo "changed $rel"
        any=true
        if $apply; then cp "$new/$rel" "$ref/$rel"; fi
        continue
    fi
    # The same masks on both images, then compare prints the count of
    # differing pixels on stderr.
    draw=()
    if [[ -f $masks ]]; then
        while read -r img rect _; do
            [[ $img == "$rel" ]] || continue
            IFS=, read -r mx my mw mh <<<"$rect"
            draw+=(-draw "rectangle $mx,$my $((mx + mw - 1)),$((my + mh - 1))")
        done < <(grep -v '^[[:space:]]*#' "$masks")
    fi
    convert "$new/$rel" -fill black "${draw[@]}" "PNG32:$tmp/new.png"
    convert "$ref/$rel" -fill black "${draw[@]}" "PNG32:$tmp/ref.png"
    diff=$(compare -metric AE -fuzz "${fuzz}%" "$tmp/new.png" "$tmp/ref.png" null: 2>&1 || true)
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
done < <(cd "$ref" && { [[ -f main-screen.png ]] && echo main-screen.png; find guide -name '*.png' -type f 2>/dev/null | sort; })

if [[ -n ${GITHUB_OUTPUT:-} ]]; then echo "changed=$any" >> "$GITHUB_OUTPUT"; fi
