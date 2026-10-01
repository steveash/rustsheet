#!/bin/bash
# SessionStart hook for Claude Code on the web: makes `make check` runnable.
# Idempotent; the container is cached after this completes.
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"

# Rust toolchain pinned by rust-toolchain.toml (rustup installs it on first use).
rustup show active-toolchain >/dev/null 2>&1 || true

# Hygiene tools used by `make ci` (skip if already present).
for tool in cargo-deny:cargo-deny cargo-machete:cargo-machete typos:typos-cli; do
  bin="${tool%%:*}"; crate="${tool##*:}"
  if ! command -v "$bin" >/dev/null 2>&1; then
    cargo install --locked --quiet "$crate" || echo "warn: could not install $crate"
  fi
done

# Python tooling (tools/) and warm Rust build caches.
if command -v uv >/dev/null 2>&1; then
  (cd tools && uv sync -q)
fi
cargo fetch --quiet
cargo build --quiet --release -p rustsheet_eval

echo "session-start: toolchain, hygiene tools, tools/ venv and eval binary ready"
