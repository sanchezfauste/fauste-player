#!/usr/bin/env bash
# Renders every raster icon in packaging/icons/ from the SVG source
# (packaging/icons/fauste-player.svg): the PNG sizes, the multi-size .ico,
# the MSIX logos (packaging/windows/msix/Assets) and the Google Play icon and
# feature graphic (packaging/android/play/en-US/images).
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
# Google Play: the 512 × 512 icon and the 1024 × 500 feature graphic (the
# icon and the name in Inter on the icon's dark gradient; no other text, so
# one graphic serves every language).
play="${root}/packaging/android/play/en-US/images"
mkdir -p "${play}"
# Play masks the icon itself, so it must fill the square: the rounded tile is
# cropped to its edges and laid on the same gradient, which fills the corners.
# The tile's thin edge stroke is left out: Play draws its own edge.
grep -v 'stroke-opacity' "$svg" > "${play}/icon-full.svg"
inkscape "${play}/icon-full.svg" --export-type=png --export-width=556 --export-height=556 \
  --export-filename="${play}/icon-full.png" >/dev/null 2>&1
magick -size 512x512 gradient:'#2a2645'-'#16151f' \
  \( "${play}/icon-full.png" -gravity center -crop 512x512+0+0 +repage \) \
  -composite -depth 8 -strip "${play}/icon.png"
rm "${play}/icon-full.svg" "${play}/icon-full.png"
render 300 "${play}/feature-icon.png"
magick -size 1024x500 gradient:'#2a2645'-'#16151f' \
  "${play}/feature-icon.png" -gravity west -geometry +90+0 -composite \
  -font "${root}/crates/fp-app/assets/fonts/Inter-600.ttf" -pointsize 84 \
  -fill '#f0eff7' -gravity west -annotate +430+0 'Fauste Player' \
  -alpha off -depth 8 -strip "${play}/featureGraphic.png"
rm "${play}/feature-icon.png"
echo "icons written to ${dir}, ${msix} and ${play}"
