#!/bin/bash
# PostToolUse hook: format the file an agent just edited (rustfmt / ruff). Never blocks.
input="$(cat)"
file="$(printf '%s' "$input" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))' 2>/dev/null)"
case "$file" in
  *.rs) rustfmt --edition 2021 --quiet "$file" 2>/dev/null || true ;;
  *.py) (cd "${CLAUDE_PROJECT_DIR:-.}/tools" && uv run --quiet ruff format --quiet "$file" 2>/dev/null) || true ;;
esac
exit 0
