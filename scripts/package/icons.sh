#!/usr/bin/env bash
# Renders every raster icon in packaging/icons/ from the SVG source
# (packaging/icons/fauste-player.svg): the PNG sizes and the multi-size .ico.
# Needs inkscape and ImageMagick (magick). Commit the regenerated files.
#
#   scripts/package/icons.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
dir="${root}/packaging/icons"
svg="${dir}/fauste-player.svg"

for tool in inkscape magick; do
  command -v "$tool" >/dev/null 2>&1 || { echo "error: '$tool' is required but not installed" >&2; exit 1; }
done
[ -f "$svg" ] || { echo "error: missing ${svg}" >&2; exit 1; }

for n in 16 32 48 64 128 256 512 1024; do
  inkscape "$svg" --export-type=png --export-width="$n" --export-height="$n" \
    --export-filename="${dir}/fauste-player-${n}.png" >/dev/null 2>&1
done

magick "${dir}/fauste-player-16.png" "${dir}/fauste-player-32.png" \
  "${dir}/fauste-player-48.png" "${dir}/fauste-player-64.png" \
  "${dir}/fauste-player-128.png" "${dir}/fauste-player-256.png" \
  "${dir}/fauste-player.ico"
echo "icons written to ${dir}"
