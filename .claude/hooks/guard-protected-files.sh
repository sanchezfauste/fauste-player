#!/usr/bin/env bash
# PreToolUse (Edit|Write|NotebookEdit): files agents never edit by hand.
f=$(jq -r '.tool_input.file_path // .tool_input.notebook_path // empty')
rel=${f#"$CLAUDE_PROJECT_DIR"/}
ask() {
  jq -n --arg r "$1" '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"ask",permissionDecisionReason:$r}}'
  exit 0
}
case "$rel" in
  CHANGELOG.md|.release-please-manifest.json)
    echo "Blocked: release-please owns $rel; never edit it by hand (CLAUDE.md, Commits and releases)." >&2
    exit 2 ;;
  docs/i18n/*)
    ask "docs/i18n is regenerated with AI before each release; feature PRs only touch it to fix moved or renamed image links." ;;
  vendor/*)
    ask "vendor/ holds patched upstream copies; a change needs its VENDORED.md updated too." ;;
esac
exit 0
