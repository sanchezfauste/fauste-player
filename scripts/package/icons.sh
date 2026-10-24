#!/usr/bin/env bash
# Renders every raster icon in packaging/icons/ from the SVG source
# (packaging/icons/fauste-player.svg): the PNG sizes, the multi-size .ico and
# the MSIX logos (packaging/windows/msix/Assets).
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
# MSIX logos: the tile and Store sizes, and the unplated taskbar sizes that
# makepri picks by target size. The wide tile centres the square icon.
msix="${root}/packaging/windows/msix/Assets"
mkdir -p "${msix}"
render() {
  inkscape "$svg" --export-type=png --export-width="$1" --export-height="$1" \
    --export-filename="$2" >/dev/null 2>&1
}
render 44 "${msix}/Square44x44Logo.png"
render 150 "${msix}/Square150x150Logo.png"
render 50 "${msix}/StoreLogo.png"
for n in 16 24 32 48 256; do
  render "$n" "${msix}/Square44x44Logo.targetsize-${n}_altform-unplated.png"
done
render 150 "${msix}/wide-icon.png"
magick -size 310x150 xc:none "${msix}/wide-icon.png" -gravity center -composite \
  "${msix}/Wide310x150Logo.png"
rm "${msix}/wide-icon.png"
echo "icons written to ${dir} and ${msix}"
