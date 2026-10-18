"""Reads stored values from an xlsx by delegating to `rustsheet-eval snapshot`.

There is exactly one xlsx value parser in the repository (Rust,
`crates/rustsheet-eval/src/xlsx_snapshot.rs`). Python tooling must not grow a
second one; it shells out to the release binary instead.
"""

from __future__ import annotations

import json
import os
import re
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


def _eval_json(*args: str) -> dict[str, Any]:
    """Runs `rustsheet-eval <args>` and parses its JSON output."""
    out = subprocess.run([str(eval_binary()), *args], check=True, capture_output=True, text=True)
    data: dict[str, Any] = json.loads(out.stdout)
    return data


def suite_cases(suite: str) -> list[dict[str, Any]]:
    """The suite's cases as the harness discovers them (`rustsheet-eval cases`, see main.rs):
    one entry per case with its `id`, case `path` and oracle `sidecar` path, and for xlsx suites
    `formula_cells`, `trusts_cached_values`, `suite_uses_cached_values` and `unreadable`."""
    out = subprocess.run(
        [str(eval_binary()), "--root", str(repo_root()), "cases", suite],
        check=True,
        capture_output=True,
        text=True,
    )
    cases: list[dict[str, Any]] = json.loads(out.stdout)
    return cases


_SHOW_FIELD = re.compile(r"^    (formula|expected|actual)\s*: ?(.*)$")
_SHOW_CELL = re.compile(r"^(\S+)\s+(\S+)$")


def parse_show(text: str) -> dict[str, dict[str, str]]:
    """Parses `rustsheet-eval show --all` output: cell key (`Sheet!A1`) → `verdict`, `formula`,
    `expected` and `actual`, each exactly as printed (numbers in Rust's shortest round-trip form,
    which `float()` reads back exactly)."""
    cells: dict[str, dict[str, str]] = {}
    current: dict[str, str] | None = None
    for line in text.splitlines()[1:]:  # the first line is the case summary
        if field := _SHOW_FIELD.match(line):
            if current is not None:
                current[field.group(1)] = field.group(2)
        elif cell := _SHOW_CELL.match(line):
            current = cells.setdefault(cell.group(1), {"verdict": cell.group(2)})
        else:
            current = None  # e.g. the `message:` line
    return cells


def show_cells(case_id: str) -> dict[str, dict[str, str]]:
    """The engine's current result for every scored cell of `case_id` (e.g.
    `probes-numeric/t1040-erfc.case.json`), from `rustsheet-eval show --all` (see `parse_show`)."""
    out = subprocess.run(
        [str(eval_binary()), "--root", str(repo_root()), "show", case_id, "--all"],
        check=True,
        capture_output=True,
        text=True,
    )
    return parse_show(out.stdout)


def snapshot(xlsx: Path) -> dict[str, Any]:
    """Returns the WorkbookSnapshot JSON (see xlsx_snapshot.rs) for an xlsx file."""
    return _eval_json("snapshot", str(xlsx))


def compare_workbooks(cached: Path, recalculated: Path) -> dict[str, Any]:
    """Per-cell verdicts of `cached`'s formula cells against `recalculated`.

    Delegates to `rustsheet-eval compare` (crates/rustsheet-eval/src/calibrate.rs), so the
    comparison rules, volatile skipping and function attribution are the harness's own.
    """
    return _eval_json("compare", str(cached), str(recalculated))


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
