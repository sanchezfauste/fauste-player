#!/bin/sh
# Builds the website into <out-dir> (default target/site): the user guide
# under guide/ and the screenshots under images/.
set -eu

root=$(cd "$(dirname "$0")/../.." && pwd)
out=${1:-$root/target/site}

mdbook=$("$root/scripts/site/mdbook.sh")

rm -rf "$out"
mkdir -p "$out"
out=$(cd "$out" && pwd)

"$mdbook" build "$root/docs" -d "$out/guide"
mkdir -p "$out/images"
cp "$root"/docs/images/* "$out/images/"

echo "Site built in $out"
