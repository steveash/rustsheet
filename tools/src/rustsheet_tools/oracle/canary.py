"""Canary: proves every oracle answer was really recalculated, inside the same workbook.

Uploading an xlsx to SharePoint/OneDrive does not recalculate it, and Microsoft documents the
Graph workbook endpoints as delegated-only, so an app-only setup that silently stopped
calculating would hand back the *cached* values we uploaded. Those look like plausible answers.

So every workbook sent to an oracle backend gets one extra trailing sheet whose formulas carry
deliberately wrong cached values. After the round trip the canary cells must hold Excel's
answers; if any poisoned value survives, that workbook was not calculated and its results are
rejected. The canary sheet is stripped before values reach a sidecar. Riding along in each
workbook costs nothing extra and covers each result individually (no separate probe run).

`embed_canary` is deliberately a package-*structure* writer (one sheet entry, relationship and
content-type override), not a value parser: values are still decoded only by the Rust snapshot
(decision 0006). Package shapes it does not recognise are refused, never guessed at.

An extra sheet is invisible to formulas except through the sheet count, so workbooks using
SHEETS() or INFO() (`INFO("numfile")`) are refused rather than answered wrongly.

Evidence for the expected values: live excel-graph runs on 2026-10-04
(docs/architecture/oracle.md, "Is Excel really calculating?").
"""

from __future__ import annotations

import io
import math
import posixpath
import re
import zipfile
from collections.abc import Callable
from dataclasses import dataclass
from html import escape
from pathlib import Path
from typing import Any

from rustsheet_tools.oracle.backends import OracleError
from rustsheet_tools.snapshot import formula_values, snapshot

SHEET = "RustSheetCanary"
_PART = "worksheets/rustsheet-canary.xml"
_REL_ID = "rIdRustSheetCanary"
_WORKSHEET_CT = "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"
_SHEET_COUNT_FUNCTIONS = re.compile(r"\b(?:_xlfn\.)?(?:SHEETS|INFO)\s*\(", re.IGNORECASE)


def _number(pred: Callable[[float], bool]) -> Callable[[Any], bool]:
    def check(value: Any) -> bool:
        if not (isinstance(value, dict) and value.get("t") == "n"):
            return False
        try:
            return pred(float(value["v"]))
        except (KeyError, TypeError, ValueError):  # malformed snapshot: a failed canary, not a crash
            return False

    return check


@dataclass(frozen=True)
class CanaryCell:
    a1: str
    formula: str
    poison: int
    """Cached value written into the uploaded file; never a value Excel could compute."""
    expect: str
    ok: Callable[[Any], bool]


CELLS = (
    CanaryCell("A1", "1+1", 999, "2", _number(lambda v: v == 2)),
    # Volatile: a fresh value in [0, 1) on every real calculation.
    CanaryCell("A2", "RAND()", 7, "a number in [0, 1)", _number(lambda v: 0 <= v < 1)),
    # Excel rounds the decimal 2.675 up (binary float rounding would give 2.67).
    CanaryCell("A3", "ROUND(2.675,2)", -1, "2.68", _number(lambda v: v == 2.68)),
    # Bit-exact: also proves the saved file keeps all 17 significant digits.
    CanaryCell("A4", "SQRT(2)", 0, repr(math.sqrt(2)), _number(lambda v: v == math.sqrt(2))),
    # The oracle must calculate as en-US (it follows the SharePoint site's regional settings):
    # month-first date text and a dot decimal separator. A day-first Excel reads "1/2/2001" as
    # 1 Feb 2001 (36923); the ironcalc workbooks were saved by one (oracle-calibration.md).
    CanaryCell("A5", 'DATEVALUE("1/2/2001")', -2, "36893 (2 Jan 2001)", _number(lambda v: v == 36893)),
    CanaryCell("A6", 'VALUE("1.5")', -3, "1.5", _number(lambda v: v == 1.5)),
)


def _sheet_xml(namespace: str) -> str:
    rows = "".join(
        f'<row r="{i}"><c r="{c.a1}"><f>{escape(c.formula, quote=False)}</f><v>{c.poison}</v></c></row>'
        for i, c in enumerate(CELLS, start=1)
    )
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        f'<worksheet xmlns="{namespace}"><sheetData>{rows}</sheetData></worksheet>'
    )


def _append_child(text: str, parent: str, child: str, part: str) -> str:
    """Inserts `<child/>` as the last child of `parent`, in the same namespace prefix (`ns0:`…)."""
    closings = list(re.finditer(rf"</((?:\w+:)?){parent}>", text))
    if not closings:
        raise OracleError(f"cannot embed the canary: no </{parent}> in {part}")
    at = closings[-1]
    return text[: at.start()] + f"<{at.group(1)}{child}/>" + text[at.start() :]


def embed_canary(xlsx: bytes) -> bytes:
    """Returns `xlsx` with the canary sheet appended after its last sheet.

    Edits the package text directly (workbook, its relationships, content types) so the case's
    own parts reach Excel byte-for-byte unchanged.
    """
    try:
        src = zipfile.ZipFile(io.BytesIO(xlsx))
        parts = {i.filename: src.read(i.filename) for i in src.infolist()}
        root_rels = parts["_rels/.rels"].decode("utf-8")
    except (KeyError, UnicodeDecodeError, zipfile.BadZipFile) as e:
        raise OracleError(f"cannot embed the canary: unreadable xlsx ({e})") from e
    office = re.search(r'Target="/?([^"]+)"[^>]*Type="[^"]*/officeDocument"', root_rels) or re.search(
        r'Type="[^"]*/officeDocument"[^>]*Target="/?([^"]+)"', root_rels
    )
    if not office or office.group(1) not in parts:
        raise OracleError("cannot embed the canary: workbook part not found")
    wb_path = office.group(1)
    wb_dir, wb_file = posixpath.split(wb_path)
    rels_path = posixpath.join(wb_dir, "_rels", f"{wb_file}.rels")
    sheet_path = posixpath.join(wb_dir, _PART)
    if rels_path not in parts or "[Content_Types].xml" not in parts or sheet_path in parts:
        raise OracleError("cannot embed the canary: unexpected package layout")

    formula_parts = (n for n in parts if n == wb_path or n.startswith(posixpath.join(wb_dir, "worksheets/")))
    for name in formula_parts:
        data = parts[name]
        if _SHEET_COUNT_FUNCTIONS.search(data.decode("utf-8", "replace")):
            raise OracleError(
                f"refusing to recalculate: {name} uses SHEETS() or INFO(), whose answer the canary "
                "sheet would change"
            )

    workbook = parts[wb_path].decode("utf-8")
    sheets = re.search(r"<((?:\w+:)?)sheets\b[^>]*>(.*?)</\1sheets>", workbook, re.S)
    rel_attr = sheets and re.search(r"\s(\w+):id=", sheets.group(2))
    if not sheets or not rel_attr:
        raise OracleError(f"cannot embed the canary: no sheets in {wb_path}")
    prefix = sheets.group(1)
    if re.search(rf'\sname="{SHEET}"', sheets.group(2), re.IGNORECASE):
        raise OracleError(f"cannot embed the canary: the workbook already has a sheet named {SHEET}")
    ns = re.search(rf'\sxmlns{":" + prefix[:-1] if prefix else ""}="([^"]+)"', workbook)
    next_id = max((int(i) for i in re.findall(r'\ssheetId="(\d+)"', sheets.group(2))), default=0) + 1
    entry = f'sheet name="{SHEET}" sheetId="{next_id}" {rel_attr.group(1)}:id="{_REL_ID}"'
    workbook = _append_child(workbook, "sheets", entry, wb_path)

    rels = parts[rels_path].decode("utf-8")
    ws_type = re.search(r'Type="([^"]*/worksheet)"', rels)
    if not ws_type or f'Id="{_REL_ID}"' in rels:
        raise OracleError(f"cannot embed the canary: unexpected relationships in {rels_path}")
    rel = f'Relationship Id="{_REL_ID}" Type="{ws_type.group(1)}" Target="{_PART}"'
    rels = _append_child(rels, "Relationships", rel, rels_path)

    types = parts["[Content_Types].xml"].decode("utf-8")
    ct = re.search(r'ContentType="([^"]*worksheet\+xml)"', types)
    override = f'Override PartName="/{sheet_path}" ContentType="{ct.group(1) if ct else _WORKSHEET_CT}"'
    types = _append_child(types, "Types", override, "[Content_Types].xml")

    parts[wb_path] = workbook.encode("utf-8")
    parts[rels_path] = rels.encode("utf-8")
    parts["[Content_Types].xml"] = types.encode("utf-8")
    parts[sheet_path] = _sheet_xml(
        ns.group(1) if ns else "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
    ).encode("utf-8")
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as zf:
        for name, data in parts.items():
            zf.writestr(name, data)
    return out.getvalue()


def canary_problems(values: dict[str, dict[str, Any]]) -> list[str]:
    """Returns one message per canary cell whose recalculated value is wrong (empty = pass)."""
    sheet = values.get(SHEET, {})
    problems = []
    for cell in CELLS:
        got = sheet.get(cell.a1)
        if not cell.ok(got):
            poisoned = got == {"t": "n", "v": float(cell.poison)}
            stale = " (the uploaded cached value: NOT recalculated)" if poisoned else ""
            problems.append(f"{cell.a1} ={cell.formula}: expected {cell.expect}, got {got}{stale}")
    return problems


def checked_values(recalculated: Path) -> dict[str, dict[str, Any]]:
    """Formula values of a recalculated workbook, minus the canary sheet; raises if the canary failed."""
    values = formula_values(snapshot(recalculated))
    problems = canary_problems(values)
    if problems:
        raise OracleError(
            "canary failed, so this workbook was not recalculated; refusing its values:\n  "
            + "\n  ".join(problems)
        )
    del values[SHEET]
    return values
