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
  * Future functions get their `_xlfn.` prefixes from XlsxWriter (use_future_functions) and, for the
    ones it does not know, from the engine exporter's list; LET/LAMBDA names are written as `_xlpm.x`.
"""

from __future__ import annotations

import argparse
import functools
import io
import json
import re
from collections.abc import Iterator
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


_IDENT = re.compile(r"[A-Za-z_][\w.]*")
_ENGINE_FUNCTIONS = Path(__file__).resolve().parents[3] / "crates/rustsheet-engine/src/functions/mod.rs"
_XLSX_NAME_ARM = re.compile(r'Function::\w+ => "(_xlfn\.[\w.]+)"\.to_string\(\)')


@functools.cache
def future_function_prefixes() -> dict[str, str]:
    """Maps bare upper-case function names to their file form (`TRIMRANGE` → `_xlfn.TRIMRANGE`).

    Read from the engine exporter's `Function::to_xlsx_string` so there is one list (T-0166).
    """
    found = _XLSX_NAME_ARM.findall(_ENGINE_FUNCTIONS.read_text(encoding="utf-8"))
    # The exporter writes the classic CEILING as `_xlfn.CEILING`, which Excel does not (T-0172);
    # a spec's CEILING must stay the plain function.
    return {f.removeprefix("_xlfn.").removeprefix("_xlws.").upper(): f for f in found if f != "_xlfn.CEILING"}


def _skip_literal(formula: str, i: int) -> int:
    """Index just past the string literal or quoted sheet name starting at `i`."""
    quote = formula[i]
    i += 1
    while i < len(formula):
        if formula[i] == quote:
            if formula[i + 1 : i + 2] == quote:
                i += 2
                continue
            return i + 1
        i += 1
    return i


def _call_args(formula: str, open_idx: int) -> tuple[list[str], int]:
    """Top-level argument texts of the call whose `(` is at `open_idx`, and the index of its `)`."""
    args: list[str] = []
    depth, start, i = 0, open_idx + 1, open_idx
    while i < len(formula):
        ch = formula[i]
        if ch in "\"'":
            i = _skip_literal(formula, i)
            continue
        if ch in "({":
            depth += 1
        elif ch in ")}":
            depth -= 1
            if depth == 0:
                args.append(formula[start:i])
                return args, i
        elif ch == "," and depth == 1:
            args.append(formula[start:i])
            start = i + 1
        i += 1
    return args, len(formula)


def _words(formula: str) -> Iterator[tuple[bool, str, int]]:
    """Splits a formula into `(is_name, text, start)` pieces; literals and punctuation are not names."""
    i = 0
    while i < len(formula):
        ch = formula[i]
        if ch in "\"'":
            end = _skip_literal(formula, i)
            yield False, formula[i:end], i
            i = end
            continue
        m = _IDENT.match(formula, i)
        if not m or (i > 0 and (formula[i - 1].isalnum() or formula[i - 1] in "_.!$")):
            yield False, ch, i
            i += 1
            continue
        yield True, m.group(0), i
        i = m.end()


def prefix_bound_names(formula: str) -> str:
    """Writes LET variables and LAMBDA parameters as `_xlpm.x`, as Excel stores them (T-0166)."""
    out: list[str] = []
    scopes: list[tuple[int, set[str]]] = []  # (end index of the call, lower-cased names)
    for is_name, word, i in _words(formula):
        end = i + len(word)
        scopes = [sc for sc in scopes if sc[0] > i]
        upper = word.upper()
        if is_name and formula[end : end + 1] == "(" and upper in ("LET", "LAMBDA"):
            args, close = _call_args(formula, end)
            decl = args[0:-1:2] if upper == "LET" else args[:-1]
            names = {a.strip().strip("[]").lower() for a in decl}
            scopes.append((close, {n for n in names if _IDENT.fullmatch(n)}))
        elif is_name and formula[end : end + 1] != "!" and any(word.lower() in names for _, names in scopes):
            out.append(f"_xlpm.{word}")
            continue
        out.append(word)
    return "".join(out)


def prefix_future_functions(formula: str) -> str:
    """Adds `_xlfn.` to calls of functions XlsxWriter does not know (e.g. TRIMRANGE)."""
    prefixes = future_function_prefixes()
    out: list[str] = []
    for is_name, word, i in _words(formula):
        if is_name and formula[i + len(word) : i + len(word) + 1] == "(" and word.upper() in prefixes:
            out.append(prefixes[word.upper()])
            continue
        out.append(word)
    return "".join(out)


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
    for name, formula in spec.get("names", {}).items():
        wb.define_name(name, formula)
    wb.close()
    return buf.getvalue()


def _write(ws: Any, row: int, col: int, raw: Any) -> None:
    if isinstance(raw, bool):
        ws.write_boolean(row, col, raw)
    elif isinstance(raw, int | float):
        ws.write_number(row, col, float(raw))
    elif isinstance(raw, str):
        if raw.startswith("="):
            ws.write_dynamic_array_formula(
                row, col, row, col, prefix_future_functions(prefix_bound_names(anchor_spill_refs(raw)))
            )
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
