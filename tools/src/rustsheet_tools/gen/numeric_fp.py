"""Policy `numeric-fp`: near-zero cancellations through M2 functions at the 8-ULP snap boundary (T-3139).

Every probe cancels X = base*(1 ± k*2^-52) against b = base (exact doubles, built as formulas on sheet
`Inputs` from powers of two as in `numeric_rules`, since Excel cuts typed literals to 15 digits) in one
structural position that docs/excel-semantics/numbers.md "Near-zero snapping" distinguishes:

- `root`: the cancelling `+`/`-` is the formula's root, its operands read through an M2 picker;
- `wrapper`: the cancellation sits under an M2 wrapper (IF, IFERROR, ROUND, ABS, `&`, `=`, AND, COUNTIF);
- `sum`: a summing function adds a range holding X and -b, with and without a trailing 0 addend.

Each position comes in both operand orders. The cases go to the M2 exit suite gen-m2 with a
`hypotheses` tag per cell (`numeric-fp/<position>/<form>/<order>/base=../sign=../k=..`); the seed
only shuffles the probes' cells. Expectations stay empty until `rustsheet-oracle fill`.
"""

from __future__ import annotations

import random
import re
from typing import Any

from rustsheet_tools.gen.generator import col_letter
from rustsheet_tools.gen.numeric_rules import BASES, fmt_k

# Four powers of two (k is a true ULP count) and 1.5 (exact; its ULP is that of 1).
FP_BASES = [b for b in BASES if b in {"1", "2", "0.5", "(2^20)", "1.5"}]
FP_K = range(10)  # straddles both boundaries: the `-` side snaps up to k = 3, the `+` side up to k = 7
SIGNS = (1, -1)
HLOOKUP_WIDTH = len(FP_K)  # one HLOOKUP block (keys k across, X and b below) per base and sign

# Inputs row r: A id, B X, C b, D:F (X, -b, 0) and G:I (-b, X, 0), the summed ranges in both orders.
X_COL, B_COL = "B", "C"
# Picker forms reading one operand; {c} is the Inputs column (B for X, C for b), {i} its 1-based offset.
PICKERS = {
    "ref": "Inputs!{c}{r}",
    "vlookup": "VLOOKUP({r},Inputs!$A$1:$C${n},{i},FALSE)",
    "hlookup": "HLOOKUP({k},Rows!$A${h}:${hl}${h2},{i},FALSE)",
    "index": "INDEX(Inputs!${c}$1:${c}${n},{r})",
    "index-match": "INDEX(Inputs!${c}$1:${c}${n},MATCH({r},Inputs!$A$1:$A${n},0))",
    "min": "MIN(Inputs!{c}{r})",
    "max": "MAX(Inputs!{c}{r})",
    "abs": "ABS(Inputs!{c}{r})",
    "round": "ROUND(Inputs!{c}{r},20)",
    "if": "IF(TRUE,Inputs!{c}{r})",
}
ORDERS = {"x-b": "{x}-{b}", "b-x": "{b}-{x}"}
WRAPPERS = {
    "if-true": "IF(TRUE,{e})",
    "iferror": "IFERROR({e},0)",
    "round-20": "ROUND({e},20)",
    "abs": "ABS({e})",
    "amp-empty": '({e})&""',
    "eq-zero": "({e})=0",
    "and-eq-zero": "AND({e}=0)",
    "countif": "COUNTIF(Inputs!F{r},{e})",
}
# Summed ranges on Inputs row r: (order, trailing-0 flag) -> columns.
SUM_RANGES = {("x-b", "no-zero"): "D{r}:E{r}", ("x-b", "zero"): "D{r}:F{r}",
              ("b-x", "no-zero"): "G{r}:H{r}", ("b-x", "zero"): "G{r}:I{r}"}  # fmt: skip
CRITERION = '"<1E+300"'  # every addend matches, so the criteria sums add the same cells as SUM
SUMMERS = {
    "sum": "SUM({rg})",
    "average": "AVERAGE({rg})",
    "sumif": f"SUMIF({{rg}},{CRITERION})",
    "sumifs": f"SUMIFS({{rg}},{{rg}},{CRITERION})",
}
CALL = re.compile(r"([A-Z][A-Z0-9.]*)\(")


def _pairs() -> list[tuple[str, int, int]]:
    """(base, sign, k) per Inputs row, in row order; HLOOKUP block j holds rows 10j+1..10j+10."""
    return [(base, sign, k) for base in FP_BASES for sign in SIGNS for k in FP_K]


def _inputs() -> tuple[dict[str, str | int], dict[str, str | int]]:
    """The `Inputs` and `Rows` (HLOOKUP blocks) sheets' cells; ids, keys and zeros are numbers, not text."""
    inputs: dict[str, str | int] = {}
    rows: dict[str, str | int] = {}
    for r, (base, sign, k) in enumerate(_pairs(), 1):
        inputs |= {
            f"A{r}": r,
            f"B{r}": f"={base}*(1{fmt_k(sign, k)})",
            f"C{r}": f"={base}",
            f"D{r}": f"=B{r}",
            f"E{r}": f"=-C{r}",
            f"F{r}": 0,
            f"G{r}": f"=-C{r}",
            f"H{r}": f"=B{r}",
            f"I{r}": 0,
        }
        top = 3 * ((r - 1) // HLOOKUP_WIDTH) + 1
        col = col_letter(k)
        rows |= {
            f"{col}{top}": k,
            f"{col}{top + 1}": f"=Inputs!B{r}",
            f"{col}{top + 2}": f"=Inputs!C{r}",
        }
    return inputs, rows


def _pick(picker: str, column: str, r: int, k: int) -> str:
    top = 3 * ((r - 1) // HLOOKUP_WIDTH) + 1
    return PICKERS[picker].format(
        c=column,
        r=r,
        k=k,
        n=len(_pairs()),
        i=2 if column == X_COL else 3,
        h=top,
        h2=top + 2,
        hl=col_letter(HLOOKUP_WIDTH - 1),
    )


def _probes(position: str) -> list[tuple[str, str]]:
    """(formula, hypothesis) for every pair and form of one position."""
    probes: list[tuple[str, str]] = []
    for r, (base, sign, k) in enumerate(_pairs(), 1):
        params = f"base={base}/sign={sign:+d}/k={k}"
        if position == "root":
            for picker in PICKERS:
                x, b = _pick(picker, X_COL, r, k), _pick(picker, B_COL, r, k)
                for order, tmpl in ORDERS.items():
                    probes.append(("=" + tmpl.format(x=x, b=b), f"{picker}/{order}/{params}"))
        elif position == "wrapper":
            for form, wrap in WRAPPERS.items():
                for order, tmpl in ORDERS.items():
                    e = tmpl.format(x=f"Inputs!{X_COL}{r}", b=f"Inputs!{B_COL}{r}")
                    probes.append(("=" + wrap.format(e=e, r=r), f"{form}/{order}/{params}"))
        else:
            for form, summer in SUMMERS.items():
                for (order, zero), rg in SUM_RANGES.items():
                    f = "=" + summer.format(rg="Inputs!" + rg.format(r=r))
                    probes.append((f, f"{form}-{zero}/{order}/{params}"))
    return probes


POSITIONS = ("root", "wrapper", "sum")


def generate_numeric_fp(seed: int) -> list[dict[str, Any]]:
    """Return one gen-m2 `rustsheet-case/v1` spec per position (5,200 probe cells in all)."""
    rng = random.Random(seed)
    inputs, rows = _inputs()
    cases: list[dict[str, Any]] = []
    for position in POSITIONS:
        probes = _probes(position)
        rng.shuffle(probes)
        calc = {f"A{i}": f for i, (f, _) in enumerate(probes, 1)}
        sheets = [{"name": "Inputs", "cells": inputs}]
        if position == "root":
            sheets.append({"name": "Rows", "cells": rows})
        sheets.append({"name": "Calc", "cells": calc})
        functions = sorted({name for f in calc.values() for name in CALL.findall(f)})
        cases.append(
            {
                "format": "rustsheet-case/v1",
                "description": f"numeric-fp probes ({position}) seed={seed}; near-zero snapping through M2 functions",
                "tags": ["generated", "M2", "numeric-fp", position, *functions],
                "sheets": sheets,
                "expected": {},
                "check": [f"Calc!{c}" for c in calc],
                "hypotheses": {
                    f"Calc!A{i}": f"numeric-fp/{position}/{t}" for i, (_, t) in enumerate(probes, 1)
                },
            }
        )
    return cases
