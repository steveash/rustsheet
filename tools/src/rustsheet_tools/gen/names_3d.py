"""Policy `names-3d`: defined names and 3-D references for M8 (suite `gen-m8a`, T-1606).

Every case is the same workbook: data sheets `Jan`, `Feb`, a blank sheet `Gap`, `Mar`, the probe
sheet `Calc`, `Apr` and `My Data` (in that tab order, so a 3-D span can cross a blank sheet and the
formula's own sheet), a second probe sheet `Other` last, and one fixed set of names: workbook and
sheet scoped (the same name in both), constants, ranges, relative references, formulas, names
inside names, LAMBDAs and 3-D ranges. Each case (one per category) differs in its seeded data and
its probes. Known behavior: docs/excel-semantics/references.md; complements the hand probes of
T-0043, T-0158, T-1459, T-1015 and T-1139 (and leaves non-ASCII names to T-1431).

No sheet name looks like a cell address (`S1` would need quotes). Probes sit in column K of
`Calc` (and `Other`, for scope), rows 1 to `PER_CATEGORY`. 3-D spans and
named ranges stay inside A1:F12, so a span over `Calc` never covers a probe (no circularity), and
the relative names (defined relative to A1, as xlsx stores them) read the Jan columns K and L,
which hold one number per probe row. Every probe yields one value (a spill would hit the next
probe). Nothing volatile. Expectations stay empty until `rustsheet-oracle fill`.
"""

from __future__ import annotations

import random
from typing import Any

from rustsheet_tools.gen.generator import col_letter

PER_CATEGORY = 280
REDRAWS = 200  # attempts to find a formula not yet in the case before accepting a repeat
PROBE_COL = "K"
DATA_ROWS = 12
DATA_COLS = 6
# Tab order matters: a 3-D span covers every sheet between its two ends.
ORDER = ["Jan", "Feb", "Gap", "Mar", "Calc", "Apr", "My Data", "Other"]
SPAN_SHEETS = ["Jan", "Feb", "Gap", "Mar", "Calc", "Apr", "My Data"]
DATA_SHEETS = ["Jan", "Feb", "Mar", "Apr", "My Data"]

# One `name formula` per line (a `Sheet!name` is sheet scoped); `#` lines are comments.
_NAMES_TEXT = """
# Constants.
k_num =42
k_neg =-1.5
k_text ="abc"
k_bool =TRUE
k_err =#N/A
k_arr ={1,2,3}
# Ranges, absolute.
r_cell =Jan!$A$1
r_block =Jan!$A$1:$C$4
r_col =Feb!$B$1:$B$12
r_row =Mar!$A$2:$F$2
r_union =(Jan!$A$1,Jan!$C$3)
r_quoted ='My Data'!$A$1:$B$3
# Ranges, relative to A1 (the stored form): read from Calc!K<i> they point at Jan near row i.
rel_same =Jan!A1
rel_right =Jan!B1
rel_down =Jan!A2
rel_col_abs =Jan!$A1
rel_row_abs =Jan!A$1
# Formulas and names inside names.
f_total =SUM(Jan!$A$1:$C$4)
f_twice =f_total*2
f_chain =f_twice+r_cell
f_text =k_text&"-"&k_num
f_count =COUNTA(r_block)
f_rel_sum =rel_same+rel_right
# LAMBDAs.
dbl =LAMBDA(x,x*2)
add =LAMBDA(a,b,a+b)
fact =LAMBDA(n,IF(n<=1,1,n*fact(n-1)))
sumsq =LAMBDA(r,SUMSQ(r))
twice_of =LAMBDA(f,x,f(f(x)))
# The same name in both scopes, and sheet-only names.
loc =10
Calc!loc =20
Jan!loc =30
Jan!only1 =5
Calc!here =Calc!$A$1
Other!here =Other!$A$1
# 3-D ranges.
span3 =Jan:Mar!$A$1:$B$2
# Quoted on one side only (valid: 'Jan:My Data'!): Excel reads it as #NAME? (references.md).
span_all =Jan:'My Data'!$A$1
span_calc =Mar:Apr!$A$1:$C$3
# Errors.
bad_ref =#REF!
bad_sheet_ref =Jan!#REF!
to_undefined =nosuch_name+1
"""
NAMES: dict[str, str] = {
    name: formula
    for name, formula in (line.split(" ", 1) for line in _NAMES_TEXT.strip().splitlines())
    if not name.startswith("#")
}

NUMBERS = ["0", "1", "2", "3", "0.5", "1.5", "10", "-2", "100", "0.25", "7"]
AGG_3D = ["SUM", "AVERAGE", "COUNT", "COUNTA", "MAX", "MIN", "PRODUCT", "AVERAGEA", "MAXA", "MINA"]
AGG_3D += ["STDEV", "STDEVP", "VAR", "VARP", "STDEV.S", "VAR.P", "SUMSQ"]
SCALAR_NAMES = ["k_num", "k_neg", "k_text", "k_bool", "k_err", "r_cell", "f_total", "f_twice", "f_chain"]
SCALAR_NAMES += ["f_text", "f_count", "loc", "here"]
RANGE_NAMES = ["r_block", "r_col", "r_row", "r_quoted", "r_cell", "k_arr"]
REL_NAMES = ["rel_same", "rel_right", "rel_down", "rel_col_abs", "rel_row_abs", "f_rel_sum"]
SPAN_NAMES = ["span3", "span_all", "span_calc"]
UNDEFINED = ["nosuch", "NoSuch2", "loc2", "only1", "Feb!loc", "Other!only1"]
DATA_POOL: list[Any] = [0, 1, -1, 2, 0.5, 3.25, 10, 100, 7, -4, "a", "1", "", True, False, None, None]
DATA_POOL += [{"t": "e", "v": "#N/A"}]

Rect = tuple[int, int, int, int]  # c0, r0, c1, r1 (0-based columns, 1-based rows)


class _Gen:
    """Seeded draws of operands over the workbook above."""

    def __init__(self, rng: random.Random) -> None:
        self.rng = rng

    def pick(self, items: list[str]) -> str:
        return self.rng.choice(items)

    def num(self) -> str:
        return self.pick(NUMBERS)

    def rect(self) -> str:
        c0, c1 = sorted(self.rng.choices(range(DATA_COLS), k=2))
        r0, r1 = sorted(self.rng.choices(range(1, DATA_ROWS + 1), k=2))
        first = f"{col_letter(c0)}{r0}"
        return first if (c0, r0) == (c1, r1) else f"{first}:{col_letter(c1)}{r1}"

    def span(self) -> str:
        """`Jan:Mar!` and its quoted forms: two distinct sheets, in either tab order."""
        a, b = self.rng.sample(SPAN_SHEETS, 2)
        if " " in a or " " in b:
            return f"'{a}:{b}'!"
        return f"{a}:{b}!"

    def cell(self) -> str:
        sheet = self.pick(DATA_SHEETS)
        prefix = f"'{sheet}'!" if " " in sheet else f"{sheet}!"
        return f"{prefix}{col_letter(self.rng.randrange(DATA_COLS))}{self.rng.randint(1, DATA_ROWS)}"


def _vary(g: _Gen, expr: str) -> str:
    """`expr` as is or inside one of a few wrappers, so categories with few base forms still
    draw distinct probes."""
    return g.pick(
        [
            expr,
            f"{expr}+{g.num()}",
            f"{expr}*{g.num()}",
            f'({expr})&"|"',
            f'IFERROR({expr},"e")',
            f"N({expr})",
        ]
    )


def _names_const(g: _Gen) -> str:
    n = g.pick(SCALAR_NAMES[:6])
    return _vary(g, g.pick([n, f"-{n}", f"{n}={g.num()}", f"ISNUMBER({n})", f"{n}>{g.num()}", f"{n}^2"]))


def _names_range(g: _Gen) -> str:
    n = g.pick(RANGE_NAMES)
    i, j = g.rng.randint(1, 4), g.rng.randint(1, 3)
    fn = g.pick(["SUM", "COUNT", "COUNTA", "MAX", "ROWS", "COLUMNS", "AVERAGE"])
    return g.pick([f"{fn}({n})", f"INDEX({n},{i},{j})", f"INDEX({n},{i})", f"{fn}({n})+{g.num()}"])


def _names_relative(g: _Gen) -> str:
    n = g.pick(REL_NAMES)
    return g.pick([n, f"{n}+{g.num()}", f"{n}*{g.num()}", f"ISBLANK({n})", f'{n}&""'])


def _names_formula(g: _Gen) -> str:
    n = g.pick(["f_total", "f_twice", "f_chain", "f_text", "f_count", "f_rel_sum"])
    other = g.pick(["k_num", "r_cell", "loc", g.num(), g.cell()])
    return g.pick([n, f"{n}+{other}", f"{n}-{other}", f"{n}&{other}", f"IF({n}>{g.num()},{other},{n})"])


def _names_lambda(g: _Gen) -> str:
    a, b = g.num(), g.pick([g.num(), g.cell(), "k_num", "r_cell"])
    return g.pick(
        [
            f"dbl({a})",
            f"dbl({b})",
            f"add({a},{b})",
            f"fact({g.rng.randint(0, 12)})",
            f"sumsq({g.cell().split('!')[0]}!{g.rect()})",
            f"twice_of(dbl,{a})",
            f"dbl({a})+add({b},{a})",
            f"add({a})",
            f"dbl({a},{b})",
            "dbl",
            f"sumsq(r_block)+{a}",
        ]
    )


def _names_scope(g: _Gen) -> str:
    return _vary(
        g,
        g.pick(
            [
                "loc",
                f"loc+{g.num()}",
                "Jan!loc",
                "Calc!loc",
                "Other!loc",
                "Jan!only1",
                f"Jan!only1*{g.num()}",
                "here",
                "Calc!here",
                "Other!here",
                f'loc&"/"&Jan!loc&"/"&{g.num()}',
                "only1",
                "Feb!loc",
            ]
        ),
    )


def _names_errors(g: _Gen) -> str:
    n = g.pick([*UNDEFINED, "bad_ref", "bad_sheet_ref", "to_undefined", "k_err"])
    return g.pick([n, f"{n}+{g.num()}", f"ISERROR({n})", f"IFERROR({n},{g.num()})", f"ERROR.TYPE({n})"])


def _three_d_aggregate(g: _Gen) -> str:
    return f"{g.pick(AGG_3D)}({g.span()}{g.rect()})"


def _three_d_mixed(g: _Gen) -> str:
    fn = g.pick(AGG_3D[:6])
    return g.pick(
        [
            f"{fn}({g.span()}{g.rect()},{g.num()})",
            f"{fn}({g.span()}{g.rect()},{g.cell()})",
            f"{fn}({g.span()}{g.rect()})+{fn}({g.span()}{g.rect()})",
            f"{fn}({g.span()}{g.rect()},{g.span()}{g.rect()})",
            f"{fn}({g.span()}{g.rect()},r_block)",
        ]
    )


def _three_d_names(g: _Gen) -> str:
    n = g.pick(SPAN_NAMES)
    return _vary(g, g.pick([f"{g.pick(AGG_3D)}({n})", f"{g.pick(AGG_3D[:4])}({n},{g.num()})", f"sumsq({n})"]))


def _three_d_misuse(g: _Gen) -> str:
    ref = g.pick([f"{g.span()}{g.rect()}", g.pick(SPAN_NAMES)])
    return g.pick(
        [
            ref,
            f"{ref}+{g.num()}",
            f"INDEX({ref},1,1)",
            f"ROWS({ref})",
            f"ISREF({ref})",
            f"ISERROR({ref})",
            f"IFERROR({ref},{g.num()})",
            f"SUM(#REF!)+{g.num()}",
        ]
    )


def _mixed(g: _Gen) -> str:
    parts = [
        g.pick(SCALAR_NAMES),
        f"{g.pick(AGG_3D[:4])}({g.span()}{g.rect()})",
        f"dbl({g.num()})",
        g.pick(REL_NAMES),
        g.cell(),
        g.num(),
    ]
    a, b = g.rng.sample(parts, 2)
    return g.pick([f"{a}+{b}", f"{a}*{b}", f"{a}&{b}", f"MAX({a},{b})", f"IF({a}>{b},{a},{b})"])


CATEGORIES = {
    "names-const": _names_const,
    "names-range": _names_range,
    "names-relative": _names_relative,
    "names-formula": _names_formula,
    "names-lambda": _names_lambda,
    "names-scope": _names_scope,
    "names-errors": _names_errors,
    "3d-aggregate": _three_d_aggregate,
    "3d-mixed": _three_d_mixed,
    "3d-names": _three_d_names,
    "3d-misuse": _three_d_misuse,
    "mixed": _mixed,
}
# Categories whose probes are also placed on `Other` (sheet scope reads differently there).
ON_OTHER = {"names-scope", "names-relative"}


def _data_cells(rng: random.Random, sheet: str) -> dict[str, Any]:
    cells: dict[str, Any] = {}
    if sheet == "Gap":
        return cells
    for r in range(1, DATA_ROWS + 1):
        for c in range(DATA_COLS):
            v = rng.choice(DATA_POOL)
            if v is not None:
                cells[f"{col_letter(c)}{r}"] = v
    if sheet == "Jan":
        # What the relative names read from probe rows 1..PER_CATEGORY (+1 for rel_down).
        for r in range(1, PER_CATEGORY + 2):
            cells[f"K{r}"] = r
            cells[f"L{r}"] = rng.choice([0.5, -1, 3, 10])
            if r > DATA_ROWS:
                cells[f"A{r}"] = r % 7
    return cells


def _probes(g: _Gen, draw: Any, count: int, seen: set[str]) -> list[str]:
    """Up to `count` distinct formulas; a draw that keeps repeating is dropped (it is the same
    probe again)."""
    formulas = []
    for _ in range(count):
        for _ in range(REDRAWS):
            formula = "=" + draw(g)
            if formula not in seen:
                seen.add(formula)
                formulas.append(formula)
                break
    return formulas


def generate_names_3d(seed: int, per_category: int = PER_CATEGORY) -> list[dict[str, Any]]:
    """Return one `rustsheet-case/v1` spec per category, `per_category` probes on each probe sheet."""
    cases: list[dict[str, Any]] = []
    for category, draw in CATEGORIES.items():
        g = _Gen(random.Random(f"{seed}/{category}"))
        sheets = {name: _data_cells(g.rng, name) for name in ORDER if name != "Other"}
        sheets["Other"] = {"A1": "other"}
        sheets["Calc"]["A1"] = "calc" if g.rng.random() < 0.5 else 1
        hypotheses: dict[str, str] = {}
        for probe_sheet in ["Calc", "Other"] if category in ON_OTHER else ["Calc"]:
            for row, formula in enumerate(_probes(g, draw, per_category, set()), start=1):
                sheets[probe_sheet][f"{PROBE_COL}{row}"] = formula
                hypotheses[f"{probe_sheet}!{PROBE_COL}{row}"] = category
        cases.append(
            {
                "format": "rustsheet-case/v1",
                "description": f"names-3d probes ({category}) seed={seed}; M8 defined names and 3-D references",
                "tags": ["generated", "M8", "names-3d", category],
                "names": dict(NAMES),
                "sheets": [{"name": name, "cells": sheets[name]} for name in ORDER],
                "expected": {},
                "check": list(hypotheses),
                "hypotheses": hypotheses,
            }
        )
    return cases
