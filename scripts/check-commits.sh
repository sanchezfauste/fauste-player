#!/usr/bin/env bash
# Checks that every commit subject in <base>..<head> follows Conventional
# Commits 1.0.0 (merge commits are skipped).
#
#   scripts/check-commits.sh <base> [head]
set -euo pipefail

base="${1:?usage: check-commits.sh <base> [head]}"
head="${2:-HEAD}"
pattern='^(feat|fix|perf|refactor|docs|test|build|ci|chore|style|revert)(\([a-z0-9._/-]+\))?!?: [^ ].*'

status=0
while IFS= read -r subject; do
  if [[ ! "${subject}" =~ ${pattern} ]]; then
    echo "not a Conventional Commit: ${subject}" >&2
    status=1
  fi
done < <(git log --no-merges --format=%s "${base}..${head}")
exit "${status}"
