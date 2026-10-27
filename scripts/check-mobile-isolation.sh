#!/usr/bin/env bash
# Guards the mobile `unsafe` exception (mobile spec §8): fails when a
# desktop crate depends on fp-mobile, when a crate other than fp-mobile
# leaves the workspace lints (`unsafe_code = "forbid"`), or when fp-mobile
# allows `unsafe` crate-wide instead of denying it.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "${root}"
status=0

# 1. No other workspace crate declares fp-mobile as a dependency (on any
# target, of any kind): fp-mobile is only ever the Android library itself.
dependents="$(cargo metadata --no-deps --format-version 1 | python3 -c '
import json, sys
meta = json.load(sys.stdin)
for package in meta["packages"]:
    if package["name"] == "fp-mobile":
        continue
    if any(d["name"] == "fp-mobile" for d in package["dependencies"]):
        print(package["name"])
')"
if [[ -n "${dependents}" ]]; then
  echo "check-mobile-isolation: these crates depend on fp-mobile: ${dependents//$'\n'/ }" >&2
  status=1
fi

# 2. Every other crate keeps the workspace lints.
for manifest in crates/*/Cargo.toml; do
  [[ "${manifest}" == crates/fp-mobile/Cargo.toml ]] && continue
  if ! awk '/^\[lints\]/{inside=1; next} /^\[/{inside=0} inside && /^workspace *= *true/{found=1} END{exit !found}' "${manifest}"; then
    echo "check-mobile-isolation: ${manifest} does not use the workspace lints" >&2
    status=1
  fi
done

# 3. fp-mobile denies unsafe code; each site opts in.
if ! grep -Eq '^unsafe_code *= *"deny"' crates/fp-mobile/Cargo.toml; then
  echo "check-mobile-isolation: fp-mobile must set unsafe_code = \"deny\"" >&2
  status=1
fi
if grep -rEn '#!\[allow\(unsafe_code\)\]' crates/fp-mobile/src >/dev/null; then
  echo "check-mobile-isolation: fp-mobile allows unsafe code crate-wide" >&2
  status=1
fi

if [[ ${status} -eq 0 ]]; then
  echo "check-mobile-isolation: ok"
fi
exit ${status}
