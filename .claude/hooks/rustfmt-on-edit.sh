#!/usr/bin/env bash
# PostToolUse (Edit|Write): format the edited Rust file, skipping vendor/.
f=$(jq -r '.tool_input.file_path // empty')
case "$f" in
  */vendor/*) exit 0 ;;
  *.rs) ;;
  *) exit 0 ;;
esac
fmt=$(command -v rustfmt || echo "$HOME/.cargo/bin/rustfmt")
"$fmt" --edition 2024 "$f" >/dev/null 2>&1
exit 0
