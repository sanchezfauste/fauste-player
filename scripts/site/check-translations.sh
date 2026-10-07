#!/bin/sh
# Usage: check-translations.sh [<site-dir>]
# Fails (exit 1) when a translation of the user guide (docs/i18n/<lang>) does
# not mirror the English guide at the commit it was translated from (same
# files, same SUMMARY entries), lacks its metadata or the AI-translation
# notice on its start page, or, given a site built by build.sh, when a
# translated page lacks the language menu and notice script.
set -eu
[ $# -le 1 ] || { echo "usage: $0 [<site-dir>]" >&2; exit 2; }
exec python3 "$(dirname "$0")/guide.py" check "$@"
