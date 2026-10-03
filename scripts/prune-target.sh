#!/usr/bin/env bash
# Frees disk space in `target/`: deletes test and example executables that no
# build has relinked for <minutes> (default 60), and incremental caches older
# than that. Libraries (.rlib, .rmeta) and build-script outputs are kept, so
# the next build only relinks. Each test binary carries its own debug info,
# and every change of features or dependencies leaves another copy behind.
#
#   scripts/prune-target.sh [minutes]
set -euo pipefail

minutes="${1:-60}"
root="$(cd "$(dirname "$0")/.." && pwd)/target"
[[ -d "${root}" ]] || exit 0

before="$(du -sh "${root}" | cut -f1)"
# Executables have no extension; everything else in deps is an input.
find "${root}" -path '*/deps/*' -type f -mmin "+${minutes}" ! -name '*.*' -delete
find "${root}" -path '*/examples/*' -type f -mmin "+${minutes}" ! -name '*.*' -delete
find "${root}" -type d -name incremental -prune -print | while IFS= read -r dir; do
  find "${dir}" -mindepth 1 -maxdepth 1 -type d -mmin "+${minutes}" -exec rm -rf {} +
done
echo "target: ${before} -> $(du -sh "${root}" | cut -f1)"
