#!/usr/bin/env bash
# Builds the release binary for one target and packs it with its notices.
#
#   scripts/package-release.sh <target> [version]
#
# Produces dist/fauste-player-<version>-<target>.tar.gz (.zip on Windows)
# and a .sha256 file next to it. The version defaults to the workspace
# version in Cargo.toml.
set -euo pipefail

target="${1:?usage: package-release.sh <target> [version]}"
version="${2:-$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n1)}"
name="fauste-player-${version}-${target}"
root="$(cd "$(dirname "$0")/.." && pwd)"
dist="${root}/dist"
stage="${dist}/${name}"

# Optional audio systems, e.g. FEATURES="jack" (see fp-app features). The
# portable archives leave out `pipewire`: it links libpipewire, which the
# binary would then need just to start (distribution packages can add it).
cargo build --release --locked --target "${target}" -p fp-app --bin fauste-player \
  ${FEATURES:+--features "${FEATURES}"}

exe="fauste-player"
case "${target}" in *windows*) exe="fauste-player.exe" ;; esac
case "${target}" in
  *linux*) "${root}/scripts/check-runtime-deps.sh" "${root}/target/${target}/release/${exe}" ;;
esac

rm -rf "${stage}"
mkdir -p "${stage}/licenses"
cp "${root}/target/${target}/release/${exe}" "${stage}/"
cp "${root}/README.md" "${root}/CHANGELOG.md" "${stage}/"
cp "${root}/crates/fp-app/assets/fonts/OFL.txt" "${stage}/licenses/Inter-OFL.txt"
[ -f "${root}/LICENSE" ] && cp "${root}/LICENSE" "${stage}/"
if command -v cargo-about >/dev/null; then
  # The notices cover exactly the crates this build links.
  cargo about generate --locked -m "${root}/crates/fp-app/Cargo.toml" -c "${root}/about.toml" \
    ${FEATURES:+--features "${FEATURES}"} \
    -o "${stage}/licenses/THIRD-PARTY.html" "${root}/about.hbs"
elif [ -n "${CI:-}" ]; then
  echo "cargo-about is required in CI" >&2
  exit 1
fi

cd "${dist}"
case "${target}" in
  *windows*)
    archive="${name}.zip"
    rm -f "${archive}"
    if command -v 7z >/dev/null; then 7z a -tzip "${archive}" "${name}" >/dev/null
    else zip -qr "${archive}" "${name}"; fi
    ;;
  *)
    archive="${name}.tar.gz"
    tar -czf "${archive}" "${name}"
    ;;
esac

if command -v sha256sum >/dev/null; then sha256sum "${archive}" > "${archive}.sha256"
else shasum -a 256 "${archive}" > "${archive}.sha256"; fi

echo "${dist}/${archive}"
