#!/usr/bin/env bash
# Lists IronCalc commits since the last triaged upstream commit (see UPSTREAM.md),
# for the upstream-watch task. Read-only; never merges.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LAST=$(grep -oE '`[0-9a-f]{40}`' "$ROOT/UPSTREAM.md" | tail -1 | tr -d '`')
DIR="${UPSTREAM_DIR:-$ROOT/target/upstream-ironcalc}"
if [ ! -d "$DIR/.git" ]; then
  git clone --quiet --filter=blob:none https://github.com/ironcalc/IronCalc.git "$DIR"
fi
git -C "$DIR" fetch --quiet origin main
git -C "$DIR" log --reverse --format='%h %ad %s' --date=short "$LAST..origin/main" -- base xlsx bindings/python
