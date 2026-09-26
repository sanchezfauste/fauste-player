#!/usr/bin/env bash
# Writes the AppStream metainfo with a <releases> entry for the version being
# packaged (release-please owns the version; the source file carries none).
#
#   scripts/package/metainfo.sh <version> <output file>
set -euo pipefail
version="${1:?usage: metainfo.sh <version> <output>}"
out="${2:?usage: metainfo.sh <version> <output>}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
date="$(date -u +%Y-%m-%d)"
sed "s|^  <!-- <releases>.*-->$|  <releases>\n    <release version=\"${version}\" date=\"${date}\"/>\n  </releases>|" \
  "${root}/packaging/linux/org.fauste.FaustePlayer.metainfo.xml" > "${out}"
