"""RustSheet developer tooling (oracle client, spec→xlsx builder, synthetic generator)."""

from __future__ import annotations

from pathlib import Path


def repo_root() -> Path:
    """Nearest ancestor of the CWD (or of this file) that contains `eval/suites`."""
    for start in (Path.cwd(), Path(__file__).resolve()):
        for candidate in (start, *start.parents):
            if (candidate / "eval" / "suites").is_dir():
                return candidate
    msg = "could not locate the repository root (no eval/suites directory found)"
    raise RuntimeError(msg)
