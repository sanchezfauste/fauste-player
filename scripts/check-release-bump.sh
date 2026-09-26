#!/usr/bin/env bash
# Simulates the version bump of a release PR with release-please's own TOML
# updater, on a scratch copy of the repository, and checks that the result
# still builds with `--locked` (Cargo.toml and every workspace crate in
# Cargo.lock must move together).
#
#   scripts/check-release-bump.sh [version]     (needs node, npm and cargo)
set -euo pipefail

version="${1:-99.0.0}"
root="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT

git -C "${root}" archive HEAD | tar -x -C "${work}"
cp "${root}/release-please-config.json" "${work}/"
(cd "${work}" && npm init -y >/dev/null && npm install --silent --no-audit --no-fund release-please@17 >/dev/null)

cat > "${work}/bump.js" <<'JS'
const fs = require('fs');
const { GenericToml } = require('release-please/build/src/updaters/generic-toml');
const { Version } = require('release-please/build/src/version');
const version = Version.parse(process.argv[2]);
const config = JSON.parse(fs.readFileSync('release-please-config.json', 'utf8'));
for (const file of config.packages['.']['extra-files']) {
  const before = fs.readFileSync(file.path, 'utf8');
  const after = new GenericToml(file.jsonpath, version).updateContent(before);
  if (after === before) {
    console.error(`not updated: ${file.path} ${file.jsonpath}`);
    process.exit(1);
  }
  fs.writeFileSync(file.path, after);
}
JS
(cd "${work}" && node bump.js "${version}")
(cd "${work}" && cargo metadata --locked --format-version 1 >/dev/null)
echo "release bump to ${version} keeps Cargo.lock consistent"
