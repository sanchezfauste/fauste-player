#!/usr/bin/env bash
# Writes THIRD-PARTY.html, the notices of the crates the app links, with
# cargo-about. Used by the builds that do not go through package-release.sh
# (the snap, the Flatpak and Flathub, the Android app).
#
#   scripts/package/notices.sh <output> [features]
#
# NOTICES_MANIFEST names the crate whose dependencies are listed, relative
# to the repository (default crates/fp-app/Cargo.toml; the Android app
# uses crates/fp-mobile/Cargo.toml).
#
# Uses cargo-about from PATH when there is one; otherwise, on Linux x86_64
# and aarch64, downloads the pinned release below and checks it first.
set -euo pipefail

output="${1:?usage: notices.sh <output> [features]}"
features="${2:-}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
manifest="${root}/${NOTICES_MANIFEST:-crates/fp-app/Cargo.toml}"

CARGO_ABOUT_VERSION=0.9.2

about="$(command -v cargo-about || true)"
if [ -z "${about}" ]; then
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) triple=x86_64-unknown-linux-musl
      sha=9099a59e820c38a68b9d65f300662a567d56562f9a10f6aa4c7e86c17c2566af ;;
    Linux-aarch64) triple=aarch64-unknown-linux-musl
      sha=af5169282fb6f84e13471493f405437e43ac517744c9ae12fbe2cdf0a6f0e5a8 ;;
    *) echo "cargo-about is required" >&2; exit 1 ;;
  esac
  tmp="$(mktemp -d)"
  trap 'rm -rf "${tmp}"' EXIT
  name="cargo-about-${CARGO_ABOUT_VERSION}-${triple}"
  curl --proto '=https' --tlsv1.2 -sSfL -o "${tmp}/about.tar.gz" \
    "https://github.com/EmbarkStudios/cargo-about/releases/download/${CARGO_ABOUT_VERSION}/${name}.tar.gz"
  echo "${sha}  ${tmp}/about.tar.gz" | sha256sum -c --quiet -
  tar -xzf "${tmp}/about.tar.gz" -C "${tmp}"
  about="${tmp}/${name}/cargo-about"
fi

mkdir -p "$(dirname "${output}")"
"${about}" about generate --locked -m "${manifest}" -c "${root}/about.toml" \
  ${features:+--features "${features}"} -o "${output}" "${root}/about.hbs"
