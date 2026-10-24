#!/usr/bin/env bash
# Writes the offline cargo sources of the Flatpak build from Cargo.lock, with
# a fixed commit of flatpak-cargo-generator checked against its SHA-256.
# Needs python3 with aiohttp and tomlkit, and curl.
#
#   scripts/package/flatpak-cargo-sources.sh [output]   # default packaging/flatpak/cargo-sources.json
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
out="${1:-${root}/packaging/flatpak/cargo-sources.json}"
commit=41c20aa10819cdb2a4f3ca171758a96d1955c018
sha256=0a2db6be87d75910facef28ab46d4d6460802e8419ab850d0caa6a364d26b380
tool="$(mktemp -d)/flatpak-cargo-generator.py"
trap 'rm -rf "$(dirname "${tool}")"' EXIT

curl -fsSL -o "${tool}" \
  "https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/${commit}/cargo/flatpak-cargo-generator.py"
echo "${sha256}  ${tool}" | sha256sum -c - >&2
python3 "${tool}" "${root}/Cargo.lock" -o "${out}"
