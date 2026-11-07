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
import sys
import tempfile
from pathlib import Path
from typing import Any

from rustsheet_tools import repo_root
from rustsheet_tools.sheetop import column_letters
from rustsheet_tools.spec2xlsx import parse_a1


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


def engine_export(case: Path) -> bytes:
    """The package an engine-export case (`.export.json`, T-4177) saves: `rustsheet-eval export`,
    the file the oracle opens in place of the case."""
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "export.xlsx"
        subprocess.run(
            [str(eval_binary()), "--root", str(repo_root()), "export", str(case), "--out", str(out)],
            check=True,
            capture_output=True,
            text=True,
        )
        return out.read_bytes()


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


def show_cells(case_id: str, binary: Path | None = None) -> dict[str, dict[str, str]]:
    """The engine's current result for every scored cell of `case_id` (e.g.
    `probes-numeric/t1040-erfc.case.json`), from `rustsheet-eval show --all` (see `parse_show`);
    `binary` runs another build of the harness instead of `eval_binary()`."""
    out = subprocess.run(
        [str(binary or eval_binary()), "--root", str(repo_root()), "show", case_id, "--all"],
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


# Same string as UNDECODED_RICH_ERROR in crates/rustsheet-eval/src/xlsx_snapshot.rs: a rich-value
# error the harness cannot decode yet. Sidecars keep it (T-2397); nothing may read it as a value.
UNDECODED = {"t": "e", "v": "#UNDECODED!"}


def is_undecoded(value: Any) -> bool:
    """Whether a sidecar value is the `UNDECODED` marker (`is_undecoded` in xlsx_snapshot.rs)."""
    return bool(value == UNDECODED)


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


# A pivot area larger than this is not captured (directive 5): its sidecar would be huge, and no
# probe needs one. The pivot is reported and left out of the sidecar.
MAX_PIVOT_AREA_CELLS = 100_000


def pivot_area(pivot: dict[str, Any]) -> tuple[int, int, int, int]:
    """(top, left, bottom, right), 0-based, of a snapshot pivot's cells: its `location` and the
    page-field area above it (`row_page_count` rows plus the blank row before the location), at
    least as wide as the location."""
    first, _, last = str(pivot["location"]).partition(":")
    top, left = parse_a1(first)
    bottom, right = parse_a1(last or first)
    pages = int(pivot.get("row_page_count") or 0)
    if pages:
        top = max(0, top - pages - 1)
        right = max(right, left + 3 * max(1, int(pivot.get("col_page_count") or 1)) - 2)
    return top, left, bottom, right


def pivot_areas(snap: dict[str, Any], texts: dict[str, dict[str, str]] | None = None) -> list[dict[str, Any]]:
    """Every pivot Excel saved in `snap` (`rustsheet-eval snapshot`'s `pivots`, T-4123) with `cells`:
    sheet cell A1 -> value for each cell of its area (`pivot_area`), `{"t": "empty"}` for a blank
    one, so an oracle sidecar records Excel's whole render. With `texts` (sheet -> A1 -> displayed
    text, the backend's `pivot_texts`), each pivot also gets `text`: A1 -> Excel's displayed text of
    each cell of its area that shows any (T-4169); a pivot with none gets no `text`. A pivot whose
    area passes `MAX_PIVOT_AREA_CELLS`, or whose location does not parse, is left out."""
    by_sheet = {s["name"]: s["cells"] for s in snap["sheets"]}
    out = []
    for pivot in snap.get("pivots", []):
        try:
            top, left, bottom, right = pivot_area(pivot)
        except (ValueError, KeyError) as e:  # an untrusted location: skip this pivot, not the fill
            print(f"PIVOT {pivot.get('sheet')}!{pivot.get('name')}: {e}, not captured", file=sys.stderr)
            continue
        if (bottom - top + 1) * (right - left + 1) > MAX_PIVOT_AREA_CELLS:
            print(f"PIVOT {pivot['sheet']}!{pivot['name']}: area too large, not captured", file=sys.stderr)
            continue
        cells = by_sheet.get(pivot["sheet"], {})
        shown = (texts or {}).get(pivot["sheet"], {})
        area, text = {}, {}
        for row in range(top, bottom + 1):
            for col in range(left, right + 1):
                a1 = f"{column_letters(col + 1)}{row + 1}"
                cell = cells.get(a1)
                area[a1] = cell["value"] if cell is not None else {"t": "empty"}
                if shown.get(a1):
                    text[a1] = shown[a1]
        out.append({**pivot, "cells": area, **({"text": text} if text else {})})
    return out


def has_libreoffice() -> bool:
    return shutil.which("soffice") is not None
