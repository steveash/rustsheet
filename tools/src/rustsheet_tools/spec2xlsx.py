"""Builds an .xlsx from a `rustsheet-case/v1` spec so a real Excel can compute it.

Conventions (normative; mirrored by the Rust engine adapter in crates/rustsheet-eval/src/engine.rs):
  * JSON number → number cell, JSON bool → boolean cell, "=..." → formula, other string → text.
  * Tagged values: {"t":"s"} text, {"t":"n"} number, {"t":"b"} bool, {"t":"e"} error.
  * Every formula is written as a *dynamic-array* formula, i.e. with the semantics of a
    formula typed into current Microsoft 365 Excel (no implicit intersection).
    A sheet may list cells in `"legacy_formulas": ["H1", ...]`; those are written as plain `<f>`
    formulas (no `cm` dynamic-array metadata), as a pre-dynamic-array file holds them (T-1077).
    The engine adapter ignores the list. `@` in such a cell is still written as raw `@`, not the
    stored `_xlfn.SINGLE`, so it shows Excel's parse of the text, not of a genuine legacy file.
  * Error literals are written as the formula `=#N/A` etc. (xlsx writers cannot emit
    error constants); they behave identically for dependants. A typed error with `"stored": true`
    (`{"t":"e","v":"#BLOCKED!","stored":true}`) is instead patched into the file as a stored
    constant `<c t="e"><v>#BLOCKED!</v></c>`, for names Excel rejects as formula grammar (T-0999).
  * The empty-string text constant "" is written as the formula `=""` (XlsxWriter turns an
    empty string into a blank cell, which has different semantics).
  * Spill references (`A1#`) are written in their file form `_xlfn.ANCHORARRAY(A1)`; Excel for the
    web rejects a literal `A1#` in `<f>` (T-0042). The engine reads the spec text, so cases keep `A1#`.
  * Typed entry `{"entered":"(1)"}` (T-0280): the cell is left blank and the docProps custom
    properties `rustsheet.entered.<n>` (one per cell, n from 1 in spec order; Excel caps a property
    value at 255 characters) hold it as JSON `[sheet, a1, text]`; oracle backends write
    the text into the cell as a user types it (Graph `values`), so Excel picks value and number
    format. The engine adapter enters the same text through its user-input parser.
  * Future functions get their `_xlfn.` prefixes from the builder's own table (the engine exporter's
    list plus `_BUILDER_ONLY_PREFIXES`), never from XlsxWriter's `use_future_functions`, which skips a
    whole formula once any call in it is prefixed (T-0052); LET/LAMBDA names are written as `_xlpm.x`.
"""

from __future__ import annotations

import argparse
import functools
import io
import json
import re
import zipfile
from collections.abc import Iterator
from datetime import datetime
from pathlib import Path
from typing import Any
from xml.sax.saxutils import escape, unescape

import xlsxwriter

# XlsxWriter stamps docProps/core.xml with the wall clock; a fixed instant keeps identical specs
# byte-identical so `input_sha256` (sidecars, eval/.cache/oracle) is stable across runs (T-0015).
_FIXED_CREATED = datetime(2000, 1, 1)

ENTERED_PROPERTY = "rustsheet.entered"
_ENTERED_XML = re.compile(rf'name="{re.escape(ENTERED_PROPERTY)}\.(\d+)"><vt:lpwstr>(.*?)</vt:lpwstr>', re.S)

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
    return {f.removeprefix("_xlfn.").removeprefix("_xlws.").upper(): f for f in found}


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


# Newer functions XlsxWriter (3.2.9) prefixes that the engine exporter's list lacks. The builder owns
# every prefix: XlsxWriter leaves a formula alone once it contains `_xlfn.`, so with its rewrite on,
# `=TRIMRANGE(A2)+DAYS(1,2)` stored a bare `DAYS`, which Excel reads as `#NAME?` (T-0052;
# docs/excel-semantics/xlsx-storage.md, "Newer functions need `_xlfn.`"). Its regexes also leave `.`
# unescaped (`NORM.DIST` matched `NORMSDIST`, T-0040), which the builder's tokenizer cannot do.
# tools/tests/test_spec2xlsx_prefixes.py checks this table still covers XlsxWriter's list.
_BUILDER_ONLY_PREFIXES = {
    name: f"_xlfn.{name}"
    for name in (
        "ANCHORARRAY",
        "DAYS",
        "FILTERXML",
        "FORECAST.LINEAR",
        "GAMMA.DIST",
        "GAUSS",
        "IMAGE",
        "IMSECH",
        "QUERYSTRING",
        "SINGLE",
        "WEBSERVICE",
    )
}


def prefix_future_functions(formula: str) -> str:
    """Writes every call of a newer function with its stored prefix (`TRIMRANGE(` → `_xlfn.TRIMRANGE(`)."""
    prefixes = {**future_function_prefixes(), **_BUILDER_ONLY_PREFIXES}
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
            "use_future_functions": False,
            "strings_to_numbers": False,
            "strings_to_formulas": False,
            "strings_to_urls": False,
        },
    )
    wb.set_properties({"created": _FIXED_CREATED})
    stored: list[tuple[int, str, str]] = []  # (sheet number, cell ref, error name)
    entered: list[tuple[str, str, str]] = []  # (sheet name, cell ref, text as typed)
    for number, sheet in enumerate(spec["sheets"], start=1):
        ws = wb.add_worksheet(sheet["name"])
        legacy = set(sheet.get("legacy_formulas", ()))
        if unknown := legacy - sheet["cells"].keys():
            msg = f"legacy_formulas names cells not in the sheet: {sorted(unknown)}"
            raise ValueError(msg)
        for a1, raw in sheet["cells"].items():
            row, col = parse_a1(a1)
            if a1 in legacy:
                if not (isinstance(raw, str) and raw.startswith("=")):
                    msg = f"legacy_formulas cell {a1} is not a formula"
                    raise ValueError(msg)
                ws.write_formula(row, col, _formula_text(raw))
                continue
            if isinstance(raw, dict) and raw.get("t") == "e" and raw.get("stored"):
                # A placeholder formula cell with an error result; `_store_errors` swaps it for the constant.
                ws.write_formula(row, col, "=0", None, "#N/A")
                stored.append((number, xlsxwriter.utility.xl_rowcol_to_cell(row, col), str(raw["v"])))
                continue
            if isinstance(raw, dict) and "entered" in raw:
                entered.append(
                    (sheet["name"], xlsxwriter.utility.xl_rowcol_to_cell(row, col), str(raw["entered"]))
                )
                continue
            _write(ws, row, col, raw)
    for name, formula in spec.get("names", {}).items():
        wb.define_name(name, formula)
    for n, entry in enumerate(entered, start=1):
        value = json.dumps(entry, ensure_ascii=False)
        if len(value) > 255:  # Excel's custom-property limit; fail here, not at the oracle
            msg = f"entered cell {entry[0]}!{entry[1]} is too long for a custom property ({len(value)} > 255)"
            raise ValueError(msg)
        wb.set_custom_property(f"{ENTERED_PROPERTY}.{n}", value)
    wb.close()
    return _store_errors(buf.getvalue(), stored) if stored else buf.getvalue()


def entered_cells(xlsx: bytes) -> list[tuple[str, str, str]]:
    """The `(sheet, a1, text)` typed entries `build_xlsx` recorded in `xlsx`, in spec order.

    Raises ValueError when the package or the property is malformed (xlsx inputs are untrusted).
    """
    try:
        with zipfile.ZipFile(io.BytesIO(xlsx)) as z:
            if "docProps/custom.xml" not in z.namelist():
                return []
            custom = z.read("docProps/custom.xml").decode("utf-8")
        # A regex, not an XML parser: the part is untrusted, and only XlsxWriter's own layout matters.
        found = sorted(
            (int(n), json.loads(unescape(v, {"&quot;": '"', "&apos;": "'"})))
            for n, v in _ENTERED_XML.findall(custom)
        )
        return [(str(s), str(a1), str(t)) for _, (s, a1, t) in found]
    except (zipfile.BadZipFile, UnicodeDecodeError, json.JSONDecodeError, TypeError, ValueError) as e:
        msg = f"malformed {ENTERED_PROPERTY} property: {e}"
        raise ValueError(msg) from e


def _store_errors(xlsx: bytes, stored: list[tuple[int, str, str]]) -> bytes:
    """Replaces the placeholder cells by stored error constants (XlsxWriter has no API for them)."""
    by_sheet: dict[str, list[tuple[str, str]]] = {}
    for number, ref, name in stored:
        by_sheet.setdefault(f"xl/worksheets/sheet{number}.xml", []).append((ref, name))
    out = io.BytesIO()
    with zipfile.ZipFile(io.BytesIO(xlsx)) as src, zipfile.ZipFile(out, "w") as dst:
        for info in src.infolist():
            data = src.read(info.filename)
            if info.filename in by_sheet:
                text = data.decode()
                for ref, name in by_sheet[info.filename]:
                    placeholder = f'<c r="{ref}" t="e"><f>0</f><v>#N/A</v></c>'
                    if text.count(placeholder) != 1:
                        msg = f"cannot store error {name} in {ref}: placeholder not found"
                        raise ValueError(msg)
                    text = text.replace(placeholder, f'<c r="{ref}" t="e"><v>{escape(name)}</v></c>')
                data = text.encode()
            dst.writestr(info, data)
    return out.getvalue()


def _formula_text(raw: str) -> str:
    """The formula as written to the file: spill refs anchored, bound names and future functions prefixed."""
    return prefix_future_functions(prefix_bound_names(anchor_spill_refs(raw)))


def _write(ws: Any, row: int, col: int, raw: Any) -> None:
    if isinstance(raw, bool):
        ws.write_boolean(row, col, raw)
    elif isinstance(raw, int | float):
        ws.write_number(row, col, float(raw))
    elif isinstance(raw, str):
        if raw.startswith("="):
            ws.write_dynamic_array_formula(row, col, row, col, _formula_text(raw))
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
