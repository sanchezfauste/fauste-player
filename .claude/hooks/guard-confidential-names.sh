#!/usr/bin/env bash
# PreToolUse (Bash): CLAUDE.md rule 2. Before a commit, a push, or a PR,
# issue or release text goes out, look for the names of other products.
#
# The list is local and never committed (the repository is public):
# $FAUSTE_FORBIDDEN_NAMES, else ~/.config/fauste-player/forbidden-names.txt.
# One name per line, matched as a whole word and case-insensitively; a line
# starting with "=" is matched case-sensitively; "#" starts a comment.
# Without a list the hook does nothing.
list=${FAUSTE_FORBIDDEN_NAMES:-$HOME/.config/fauste-player/forbidden-names.txt}
[ -r "$list" ] || exit 0

cmd=$(jq -r '.tool_input.command // empty')
case "$cmd" in
  *"git commit"* | *"git push"* | *"gh pr "* | *"gh issue "* | *"gh release "* | *"gh api "*) ;;
  *) exit 0 ;;
esac

cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
tmp=$(mktemp -d) || exit 0
trap 'rm -rf "$tmp"' EXIT

grep -v '^[[:space:]]*\(#\|$\)' "$list" | grep -v '^=' > "$tmp/any-case"
grep '^=' "$list" | sed 's/^=//' > "$tmp/exact-case"

{
  printf '%s\n' "$cmd"
  case "$cmd" in
    *"git commit"*)
      git diff --cached -U0 | grep '^+' ;;
  esac
  case "$cmd" in
    *"git push"* | *"gh pr "*)
      base=$(git merge-base HEAD origin/master 2>/dev/null) && {
        git log --format=%B "$base"..HEAD
        git diff -U0 "$base" HEAD | grep '^+'
      } ;;
  esac
  # Text handed over in a file (gh --body-file, git commit -F).
  for f in $(printf '%s\n' "$cmd" | grep -oE '(--body-file|--file|-F)[ =][^ ;&|]+' | sed -E 's/^[^ =]+[ =]//'); do
    [ -r "$f" ] && cat "$f"
  done
} > "$tmp/text" 2>/dev/null

hits=$( {
  [ -s "$tmp/any-case" ] && grep -o -i -w -F -f "$tmp/any-case" "$tmp/text"
  [ -s "$tmp/exact-case" ] && grep -o -w -F -f "$tmp/exact-case" "$tmp/text"
} | sort -u | paste -sd, -)

if [ -n "$hits" ]; then
  echo "Blocked by CLAUDE.md rule 2 (confidentiality): the text or diff names another product ($hits). Describe the behaviour in its own terms and try again." >&2
  exit 2
fi
exit 0
