#!/usr/bin/env bash
# Writes the AUR PKGBUILD of fauste-player-bin for a published release, from
# packaging/aur/PKGBUILD.in, with the checksums of the release's Linux
# archives (their .sha256 assets) and of the desktop entry and icon at the
# tag. Needs curl.
#
#   scripts/package/aur.sh <version> <output file>
set -euo pipefail

version="${1:?usage: aur.sh <version> <output>}"
out="${2:?usage: aur.sh <version> <output>}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
repo="${GITHUB_REPOSITORY:-sanchezfauste/fauste-player}"
release="https://github.com/${repo}/releases/download/v${version}"
raw="https://raw.githubusercontent.com/${repo}/v${version}"

asset_sha() {
  curl -fsSL "${release}/fauste-player-${version}-$1-unknown-linux-gnu.tar.gz.sha256" | cut -d' ' -f1
}
file_sha() {
  curl -fsSL "${raw}/$1" | sha256sum | cut -d' ' -f1
}

x86="$(asset_sha x86_64)"
arm="$(asset_sha aarch64)"
desktop="$(file_sha packaging/linux/org.fauste.FaustePlayer.desktop)"
svg="$(file_sha packaging/icons/fauste-player.svg)"
for sum in "${x86}" "${arm}" "${desktop}" "${svg}"; do
  [[ "${sum}" =~ ^[0-9a-f]{64}$ ]] || { echo "bad checksum '${sum}'" >&2; exit 1; }
done
sed -e "s/@VERSION@/${version}/" -e "s/@SHA256_X86_64@/${x86}/" -e "s/@SHA256_AARCH64@/${arm}/" \
  -e "s/@SHA256_DESKTOP@/${desktop}/" -e "s/@SHA256_SVG@/${svg}/" \
  "${root}/packaging/aur/PKGBUILD.in" > "${out}"
