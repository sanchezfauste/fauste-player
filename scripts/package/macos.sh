#!/usr/bin/env bash
# Builds "Fauste Player.app" as a universal binary (Apple silicon and Intel)
# and packs it in a .dmg with a .sha256, in dist/. Runs on macOS.
#
#   scripts/package/macos.sh [version]
#
# PREBUILT_DIR, when set, holds binaries already built for this release as
# <dir>/macos-binary-<target>/fauste-player (the CI build artefacts); they
# are used instead of compiling that architecture again.
#
# Signing and notarisation run when their secrets are set:
#   APPLE_CERT_P12 (base64 of a Developer ID Application .p12),
#   APPLE_CERT_PASSWORD, and for notarisation APPLE_ID, APPLE_TEAM_ID,
#   APPLE_APP_PASSWORD (an app-specific password).
# Without them the app is unsigned: Gatekeeper then asks to confirm the
# first start (see the user guide).
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
version="${1:-$(sed -n 's/^version = "\(.*\)"$/\1/p' "${root}/Cargo.toml" | head -n1)}"
export FEATURES="${FEATURES-jack}"
dist="${root}/dist"
work="${dist}/staging/macos"
targets=(aarch64-apple-darwin x86_64-apple-darwin)
app="${work}/Fauste Player.app"
dmg="${dist}/fauste-player-${version}-macos-universal.dmg"

rm -rf "${work}"
mkdir -p "${work}" "${app}/Contents/MacOS" "${app}/Contents/Resources/licenses"

# 1. One binary for both architectures.
binaries=()
for target in "${targets[@]}"; do
  prebuilt="${PREBUILT_DIR:-}/macos-binary-${target}/fauste-player"
  if [ -n "${PREBUILT_DIR:-}" ] && [ -f "${prebuilt}" ]; then
    # Artefacts lose the executable bit.
    chmod +x "${prebuilt}"
    binaries+=("${prebuilt}")
    continue
  fi
  rustup target add "${target}" >/dev/null
  (cd "${root}" && cargo build --release --locked --target "${target}" -p fp-app \
    --bin fauste-player ${FEATURES:+--features "${FEATURES}"})
  binaries+=("${root}/target/${target}/release/fauste-player")
done
lipo -create -output "${app}/Contents/MacOS/fauste-player" "${binaries[@]}"
"${app}/Contents/MacOS/fauste-player" --version

# 2. Bundle metadata, icon and notices.
sed "s/@VERSION@/${version}/g" "${root}/packaging/macos/Info.plist" > "${app}/Contents/Info.plist"
iconset="${work}/fauste-player.iconset"
mkdir -p "${iconset}"
for n in 16 32 128 256 512; do
  cp "${root}/packaging/icons/fauste-player-${n}.png" "${iconset}/icon_${n}x${n}.png"
  double=$((n * 2))
  cp "${root}/packaging/icons/fauste-player-${double}.png" "${iconset}/icon_${n}x${n}@2x.png"
done
iconutil -c icns -o "${app}/Contents/Resources/fauste-player.icns" "${iconset}"
cp "${root}/crates/fp-app/assets/fonts/OFL.txt" "${app}/Contents/Resources/licenses/Inter-OFL.txt"
cp "${root}/crates/fp-app/assets/licenses/Phosphor-MIT.txt" "${app}/Contents/Resources/licenses/Phosphor-MIT.txt"
if command -v cargo-about >/dev/null; then
  (cd "${root}" && cargo about generate --locked -m crates/fp-app/Cargo.toml -c about.toml \
    ${FEATURES:+--features "${FEATURES}"} \
    -o "${app}/Contents/Resources/licenses/THIRD-PARTY.html" about.hbs)
fi

# 3. Signing (hardened runtime, for notarisation).
identity=""
if [ -n "${APPLE_CERT_P12:-}" ]; then
  keychain="${work}/signing.keychain-db"
  password="$(uuidgen)"
  security create-keychain -p "${password}" "${keychain}"
  security set-keychain-settings -lut 3600 "${keychain}"
  security unlock-keychain -p "${password}" "${keychain}"
  printf '%s' "${APPLE_CERT_P12}" | base64 --decode > "${work}/cert.p12"
  security import "${work}/cert.p12" -k "${keychain}" -P "${APPLE_CERT_PASSWORD:-}" \
    -T /usr/bin/codesign
  rm -f "${work}/cert.p12"
  security set-key-partition-list -S apple-tool:,apple: -s -k "${password}" "${keychain}" >/dev/null
  # shellcheck disable=SC2046 # one word per keychain path, as listed
  security list-keychains -d user -s "${keychain}" $(security list-keychains -d user | tr -d '"')
  identity="$(security find-identity -v -p codesigning "${keychain}" \
    | sed -n 's/.*"\(Developer ID Application:.*\)"/\1/p' | head -n1)"
  codesign --force --options runtime --timestamp --sign "${identity}" \
    "${app}/Contents/MacOS/fauste-player"
  codesign --force --options runtime --timestamp --sign "${identity}" "${app}"
  codesign --verify --strict --verbose=2 "${app}"
else
  # Unsigned release: an ad-hoc signature seals the bundle, so macOS offers
  # to open it (Privacy & Security → Open Anyway) instead of calling it damaged.
  codesign --force --deep --sign - "${app}"
fi

# 4. Disk image with an Applications link.
image="${work}/image"
mkdir -p "${image}"
cp -R "${app}" "${image}/"
ln -s /Applications "${image}/Applications"
rm -f "${dmg}"
hdiutil create -volname "Fauste Player" -srcfolder "${image}" -ov -format UDZO "${dmg}" >/dev/null

# 5. Notarisation.
if [ -n "${identity}" ]; then
  codesign --force --timestamp --sign "${identity}" "${dmg}"
  if [ -n "${APPLE_ID:-}" ] && [ -n "${APPLE_TEAM_ID:-}" ] && [ -n "${APPLE_APP_PASSWORD:-}" ]; then
    xcrun notarytool submit "${dmg}" --apple-id "${APPLE_ID}" --team-id "${APPLE_TEAM_ID}" \
      --password "${APPLE_APP_PASSWORD}" --wait
    xcrun stapler staple "${dmg}"
  fi
fi

(cd "${dist}" && shasum -a 256 "$(basename "${dmg}")" > "$(basename "${dmg}").sha256")
echo "${dmg}"
