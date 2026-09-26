#!/usr/bin/env bash
# Fails if a Linux binary links a shared library beyond the ones every
# supported desktop has. Optional audio systems (JACK) must be loaded at
# run time, so a machine without them still starts the app.
#
#   scripts/check-runtime-deps.sh <binary>
set -euo pipefail

binary="${1:?usage: check-runtime-deps.sh <binary>}"
allowed='^(libasound\.so\.2|libdbus-1\.so\.3|libgcc_s\.so\.1|libm\.so\.6|libc\.so\.6|libpthread\.so\.0|libdl\.so\.2|librt\.so\.1|ld-linux[-a-z0-9_.]*\.so\.[0-9]+)$'

needed="$(LC_ALL=C readelf -d "${binary}" | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p')"
unexpected="$(printf '%s\n' "${needed}" | grep -Ev "${allowed}" || true)"
if [ -n "${unexpected}" ]; then
  echo "${binary} links libraries it would need just to start:" >&2
  printf '  %s\n' ${unexpected} >&2
  exit 1
fi
echo "runtime libraries: $(printf '%s ' ${needed})"
