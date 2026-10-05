"""Builds an .xlsx from a `rustsheet-case/v1` spec so a real Excel can compute it.

Conventions (normative; mirrored by the Rust engine adapter in crates/rustsheet-eval/src/engine.rs):
  * JSON number → number cell, JSON bool → boolean cell, "=..." → formula, other string → text.
  * Tagged values: {"t":"s"} text, {"t":"n"} number, {"t":"b"} bool, {"t":"e"} error.
  * Every formula is written as a *dynamic-array* formula, i.e. with the semantics of a
    formula typed into current Microsoft 365 Excel (no implicit intersection).
  * Error literals are written as the formula `=#N/A` etc. (xlsx writers cannot emit
    error constants); they behave identically for dependants.
  * The empty-string text constant "" is written as the formula `=""` (XlsxWriter turns an
    empty string into a blank cell, which has different semantics).
  * Spill references (`A1#`) are written in their file form `_xlfn.ANCHORARRAY(A1)`; Excel for the
    web rejects a literal `A1#` in `<f>` (T-0042). The engine reads the spec text, so cases keep `A1#`.
  * Future functions get their `_xlfn.` prefixes automatically (XlsxWriter use_future_functions).
"""

from __future__ import annotations

import argparse
import io
import json
import re
from datetime import datetime
from pathlib import Path
from typing import Any

import xlsxwriter

# XlsxWriter stamps docProps/core.xml with the wall clock; a fixed instant keeps identical specs
# byte-identical so `input_sha256` (sidecars, eval/.cache/oracle) is stable across runs (T-0015).
_FIXED_CREATED = datetime(2000, 1, 1)

_A1 = re.compile(r"^\$?([A-Za-z]{1,3})\$?(\d+)$")


# `A1#`, `$A$1#`, `Sheet!A1#`, `'My Sheet'!A1#`; the lookbehind keeps names like `X1A1#` whole.
_SPILL_REF = re.compile(r"(?<![\w.$!'])((?:(?:'(?:[^']|'')+'|[A-Za-z_][\w.]*)!)?\$?[A-Za-z]{1,3}\$?\d+)#")
_STRING_LITERAL = re.compile(r'"(?:[^"]|"")*"')


def anchor_spill_refs(formula: str) -> str:
    """Rewrites spill references outside string literals to `_xlfn.ANCHORARRAY(ref)`."""
    out: list[str] = []
    pos = 0
    for lit in _STRING_LITERAL.finditer(formula):
        out.append(_SPILL_REF.sub(r"_xlfn.ANCHORARRAY(\1)", formula[pos : lit.start()]))
        out.append(lit.group(0))
        pos = lit.end()
    out.append(_SPILL_REF.sub(r"_xlfn.ANCHORARRAY(\1)", formula[pos:]))
    return "".join(out)


def parse_a1(a1: str) -> tuple[int, int]:
    """'B3' → (2, 1) zero-based (row, col)."""
    m = _A1.match(a1.strip())
    if not m:
        msg = f"bad cell address {a1!r}"
        raise ValueError(msg)
    col = 0
    for ch in m.group(1).upper():
        col = col * 26 + (ord(ch) - ord("A") + 1)
    return int(m.group(2)) - 1, col - 1


def build_xlsx(spec: dict[str, Any]) -> bytes:
    """Returns xlsx bytes for a case spec."""
    buf = io.BytesIO()
    wb = xlsxwriter.Workbook(
        buf,
        {
            "in_memory": True,
            "use_future_functions": True,
            "strings_to_numbers": False,
            "strings_to_formulas": False,
            "strings_to_urls": False,
        },
    )
    wb.set_properties({"created": _FIXED_CREATED})
    for sheet in spec["sheets"]:
        ws = wb.add_worksheet(sheet["name"])
        for a1, raw in sheet["cells"].items():
            row, col = parse_a1(a1)
            _write(ws, row, col, raw)
    wb.close()
    return buf.getvalue()


def _write(ws: Any, row: int, col: int, raw: Any) -> None:
    if isinstance(raw, bool):
        ws.write_boolean(row, col, raw)
    elif isinstance(raw, int | float):
        ws.write_number(row, col, float(raw))
    elif isinstance(raw, str):
        if raw.startswith("="):
            ws.write_dynamic_array_formula(row, col, row, col, anchor_spill_refs(raw))
        elif raw == "":
            ws.write_formula(row, col, '=""')
        else:
            ws.write_string(row, col, raw)
    elif isinstance(raw, dict):
        tag, value = raw.get("t"), raw.get("v")
        if tag == "s" and value == "":
            ws.write_formula(row, col, '=""')
        elif tag == "s":
            ws.write_string(row, col, str(value))
        elif tag == "n":
            ws.write_number(row, col, float(value))  # type: ignore[arg-type]
        elif tag == "b":
            ws.write_boolean(row, col, bool(value))
        elif tag == "e":
            ws.write_formula(row, col, f"={value}")
        elif tag == "empty":
            return
        else:
            msg = f"unknown typed cell {raw!r}"
            raise ValueError(msg)
    else:
        msg = f"unsupported cell input {raw!r}"
        raise TypeError(msg)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("spec", type=Path)
    ap.add_argument("-o", "--out", type=Path)
    args = ap.parse_args()
    spec = json.loads(args.spec.read_text())
    out = args.out or args.spec.with_name(args.spec.name.removesuffix(".case.json") + ".xlsx")
    out.write_bytes(build_xlsx(spec))
    print(out)


if __name__ == "__main__":
    main()
