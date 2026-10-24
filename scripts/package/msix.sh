#!/usr/bin/env bash
# Builds the MSIX package for the Microsoft Store for one target, in dist/,
# and prints its path as the last line of stdout. Runs in Git Bash on
# Windows; needs MakeAppx and MakePri from the Windows SDK. The package is
# not signed: the Store signs what it publishes.
#
#   scripts/package/msix.sh <target> [version]
#
# The Store identity comes from Partner Center (Product management, Product
# identity) through MSSTORE_IDENTITY_NAME, MSSTORE_PUBLISHER and
# MSSTORE_PUBLISHER_DISPLAY_NAME. Without them the package gets placeholder
# values: it still builds (dry runs), but the Store would reject it.
set -euo pipefail

target="${1:?usage: msix.sh <target> [version]}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
version="${2:-$(sed -n 's/^version = "\(.*\)"$/\1/p' "${root}/Cargo.toml" | head -n1)}"
export FEATURES="${FEATURES-jack}"
dist="${root}/dist"
stage="${dist}/fauste-player-${version}-${target}"
work="${dist}/staging/msix-${target}"

case "${target}" in
  x86_64-*) arch=x64 ;;
  aarch64-*) arch=arm64 ;;
  *) echo "no MSIX for ${target}" >&2; exit 1 ;;
esac

# MSIX versions are four numbers and the Store needs the last one to be 0;
# a SemVer pre-release (0.2.0-rc.1) keeps only its numbers.
core="${version%%-*}"
[[ "${core}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "bad version ${version}" >&2; exit 1; }
msix_version="${core}.0"

sdk_tool() {
  command -v "$1" 2>/dev/null && return
  ls "/c/Program Files (x86)/Windows Kits/10/bin/"*/x64/"$1.exe" 2>/dev/null | sort -V | tail -n1
}
makeappx="$(sdk_tool makeappx)"
makepri="$(sdk_tool makepri)"
[ -n "${makeappx}" ] && [ -n "${makepri}" ] || { echo "MakeAppx and MakePri are required" >&2; exit 1; }

# Always rebuilt (a no-op when nothing changed): never a stale binary.
"${root}/scripts/package-release.sh" "${target}" "${version}" >/dev/null
"${root}/scripts/check-windows-gui.sh" "${stage}/fauste-player.exe" >&2

rm -rf "${work}"
mkdir -p "${work}"
cp "${stage}/fauste-player.exe" "${stage}/LICENSE" "${stage}/NOTICE" "${work}/"
cp -r "${stage}/licenses" "${work}/licenses"
cp -r "${root}/packaging/windows/msix/Assets" "${work}/Assets"

resources=""
for dir in "${root}"/crates/fp-app/locales/*/; do
  resources+="    <Resource Language=\"$(basename "${dir}")\" />"$'\n'
done
# en-US first: it is the package's default language.
resources="    <Resource Language=\"en-US\" />"$'\n'"$(printf '%s' "${resources}" | grep -v '"en-US"')"

xml_escape() { sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' -e 's/"/\&quot;/g'; }
identity="$(printf '%s' "${MSSTORE_IDENTITY_NAME:-FaustePlayer.DryRun}" | xml_escape)"
publisher="$(printf '%s' "${MSSTORE_PUBLISHER:-CN=00000000-0000-0000-0000-000000000000}" | xml_escape)"
publisher_name="$(printf '%s' "${MSSTORE_PUBLISHER_DISPLAY_NAME:-Marc Sánchez Fauste}" | xml_escape)"
# Plain bash substitution (no `&` expansion in the replacement), so the
# script needs no interpreter on the runner.
shopt -u patsub_replacement 2>/dev/null || true
manifest="$(cat "${root}/packaging/windows/msix/AppxManifest.xml")"
manifest="${manifest//@IDENTITY_NAME@/${identity}}"
manifest="${manifest//@PUBLISHER_DISPLAY_NAME@/${publisher_name}}"
manifest="${manifest//@PUBLISHER@/${publisher}}"
manifest="${manifest//@VERSION@/${msix_version}}"
manifest="${manifest//@ARCH@/${arch}}"
manifest="${manifest//@RESOURCES@/${resources}}"
printf '%s\n' "${manifest}" > "${work}/AppxManifest.xml"

# resources.pri lets Windows pick the logo by scale and target size.
# Git Bash would rewrite the /cf, /pr... switches as paths.
(cd "${work}" && MSYS_NO_PATHCONV=1 "${makepri}" createconfig /cf priconfig.xml /dq en-US /o >&2 \
  && MSYS_NO_PATHCONV=1 "${makepri}" new /pr . /cf priconfig.xml /of resources.pri /o >&2 \
  && rm priconfig.xml)

msix="${dist}/fauste-player-${version}-${target}.msix"
rm -f "${msix}"
MSYS_NO_PATHCONV=1 "${makeappx}" pack /o /h SHA256 /d "$(cygpath -w "${work}")" /p "$(cygpath -w "${msix}")" >&2
(cd "${dist}" && sha256sum "$(basename "${msix}")" > "$(basename "${msix}").sha256")
echo "${msix}"
