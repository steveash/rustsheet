"""Reads stored values from an xlsx by delegating to `rustsheet-eval snapshot`.

There is exactly one xlsx value parser in the repository (Rust,
`crates/rustsheet-eval/src/xlsx_snapshot.rs`). Python tooling must not grow a
second one; it shells out to the release binary instead.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any

from rustsheet_tools import repo_root


def eval_binary() -> Path:
    override = os.environ.get("RUSTSHEET_EVAL_BIN")
    if override:
        return Path(override)
    root = repo_root()
    candidate = root / "target" / "release" / "rustsheet-eval"
    if not candidate.exists():
        subprocess.run(
            ["cargo", "build", "-q", "--release", "-p", "rustsheet_eval"],
            cwd=root,
            check=True,
        )
    return candidate


def snapshot(xlsx: Path) -> dict[str, Any]:
    """Returns the WorkbookSnapshot JSON (see xlsx_snapshot.rs) for an xlsx file."""
    out = subprocess.run(
        [str(eval_binary()), "snapshot", str(xlsx)],
        check=True,
        capture_output=True,
        text=True,
    )
    data: dict[str, Any] = json.loads(out.stdout)
    return data


def formula_values(snap: dict[str, Any]) -> dict[str, dict[str, Any]]:
    """Maps sheet → A1 → value for every formula cell (incl. array members) in a snapshot."""
    result: dict[str, dict[str, Any]] = {}
    for sheet in snap["sheets"]:
        cells: dict[str, Any] = {}
        for address, cell in sheet["cells"].items():
            if cell.get("formula_kind") is None:
                continue
            cells[address] = cell["value"]
        result[sheet["name"]] = cells
    return result


def has_libreoffice() -> bool:
    return shutil.which("soffice") is not None
