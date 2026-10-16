"""Policy `operators-refs`: operator and reference probes for M1 (suite `gen-m1`, T-0117).

Only literals, operators and references, never a function name, so the suite measures M1 scope
(docs/milestones.md): number literals, unary minus vs `^`, `%`, `&`, comparisons across types,
text/number/bool coercion, error propagation, the range/union/intersection operators, `$`/mixed
refs, whole rows and columns, cross-sheet and quoted sheet names, and recalculation chains. Known
behavior of these constructs: docs/excel-semantics/syntax.md, coercion.md, numbers.md and
references.md. Dense floating-point rules are the `numeric-rules` policy's job (probes-numeric).

One case per category; every probe is one cell on sheet `Calc`, tagged `category/form` in the
case's `hypotheses` map, and no formula repeats within a case (a repeat is the same probe again).
Every probe yields one value: a multi-cell result would spill into the next probe (spec formulas
are dynamic-array formulas, decision 0007), so every intersection is built to cover one cell or
none, and a bare range is a single cell. Whole rows and columns appear only as intersection
operands, which never build million-cell work. Nothing volatile, and no date, currency or
thousands-separator text; the en-US decimal point is assumed (the oracle runs en-US).
Expectations stay empty until `rustsheet-oracle fill` (T-0109).

Each category is a table of `form: template` lines. In a template, a name the category binds
(`{a}`, `{rect}`) is fixed for the probe; any other name (`{num}`, `{cell}`) is a fresh draw from
`DRAWS` at each occurrence, made only for the chosen form.
"""

from __future__ import annotations

import random
import re
from collections.abc import Callable
from typing import Any

from rustsheet_tools.gen.common import SeededDraws
from rustsheet_tools.gen.generator import col_letter
from rustsheet_tools.gen.numeric_rules import CMP_OPS

PER_CATEGORY = 250
REDRAWS = 100  # attempts to find a formula not yet in the case before accepting a repeat
DATA_ROWS = 12
DATA_COLS = 5
# Quoted names: a space, and an apostrophe (doubled inside the quotes).
SHEETS = {"Data": "Data!", "My Data": "'My Data'!", "Q'1": "'Q''1'!"}

NUMBERS = ["0", "1", "2", "3", "0.5", "1.5", "10", "100", "0.1", "1E+3", "2.5", "7"]
# Non-canonical spellings Excel's literal parser accepts (it stores them normalized).
LITERALS = [".5", "1.", "007", "1e3", "1E+0", "0.0", "1.50", "2E-3", "1E+15", "123456789012345", "0.000001"]
TEXTS = ['"a"', '"A"', '""', '"abc"', '"1"', '" 2 "', '"1e3"', '"0.5"', '"-1"', '"TRUE"', '"x y"']
ERRORS = ["#DIV/0!", "#N/A", "#VALUE!", "#NUM!", "#REF!", "#NAME?", "#NULL!"]
ARITH = ["+", "-", "*", "/"]
# Data-sheet values: numbers, text (numeric-looking too), bools, errors and blanks (None).
DATA_POOL: list[Any] = [0, 1, -1, 2, 0.5, 3.25, 100, 1e-15, "a", "ABC", "1", " 2 ", "", True, False]
DATA_POOL += [{"t": "e", "v": "#N/A"}, {"t": "e", "v": "#DIV/0!"}, None, None]

Rect = tuple[int, int, int, int]  # c0, r0, c1, r1 (0-based columns, 1-based rows)


class _Gen(SeededDraws):
    """Random operands over the three data sheets."""

    def cell(self, sheet: str = "Data", dollars: bool = True) -> str:
        c, r = self.rng.randrange(DATA_COLS), self.rng.randint(1, DATA_ROWS)
        dc = "$" if dollars and self.rng.random() < 0.3 else ""
        dr = "$" if dollars and self.rng.random() < 0.3 else ""
        return f"{SHEETS[sheet]}{dc}{col_letter(c)}{dr}{r}"

    def scalar(self) -> str:
        """Any operand: a literal of any type or a cell on any data sheet."""
        roll = self.rng.random()
        if roll < 0.3:
            return self.pick(NUMBERS)
        if roll < 0.45:
            return self.pick(TEXTS)
        if roll < 0.55:
            return self.pick(["TRUE", "FALSE"])
        if roll < 0.62:
            return self.pick(ERRORS)
        return self.cell(self.pick(list(SHEETS)))

    def number(self) -> str:
        return self.pick(NUMBERS) if self.rng.random() < 0.6 else self.cell()

    def rect(self) -> Rect:
        c0, c1 = sorted(self.rng.sample(range(DATA_COLS), 2))
        r0, r1 = sorted(self.rng.sample(range(1, DATA_ROWS + 1), 2))
        return c0, r0, c1, r1

    def expr(self, depth: int = 0) -> str:
        """A random operator tree over scalars, at most 3 levels deep."""
        roll = self.rng.random()
        if depth >= 3 or roll < 0.35:
            return self.scalar()
        if roll < 0.45:
            return f"-{self.expr(depth + 1)}"
        if roll < 0.5:
            return f"{self.expr(depth + 1)}%"
        op = self.pick([*ARITH, "^", "&", *CMP_OPS])
        return f"({self.expr(depth + 1)}{op}{self.expr(depth + 1)})"


DRAWS: dict[str, Callable[[_Gen], str]] = {
    "num": _Gen.number,
    "n": lambda g: g.pick(NUMBERS),
    "lit": lambda g: g.pick(LITERALS),
    "text": lambda g: g.pick(TEXTS),
    "bool": lambda g: g.pick(["TRUE", "FALSE"]),
    "err": lambda g: g.pick(ERRORS),
    "cell": lambda g: g.cell(),
    "anycell": lambda g: g.cell(g.pick(list(SHEETS))),
    "scalar": _Gen.scalar,
    "arith": lambda g: g.pick(ARITH),
    "cmp": lambda g: g.pick(CMP_OPS),
    "exp": lambda g: g.pick(["0", "1", "2", "3", "0.5"]),
    "small": lambda g: g.pick(["2", "3", "0.5"]),
    "base": lambda g: g.pick(["2", "10", "0.5"]),
    "half": lambda g: g.pick(["2", "0.5"]),
    "sfx": lambda g: g.pick(["", "*2", "+1", "^2"]),
    "trap": lambda g: g.pick(["0.1+0.2", "1/3", "1E+20", "-0", "2/3*3", "1E-5"]),
    "blankish": lambda g: g.pick(["0", '""', "FALSE"]),
    "zero": lambda g: g.pick(["0", '"0"', "FALSE", "Data!$Z$99"]),
    "beyond": lambda g: g.pick(["$Z$99", "F1", "A13", "XFD1048576"]),
}

TEMPLATES: dict[str, str] = {
    "literal": """
bare: {a}
neg: -{a}
arith: {a}{arith}{lit}
concat: {a}&""
cmp: {a}{cmp}{n}
percent: {a}%
""",
    "unary-power": """
neg-pow: -{a}^{b}
paren-neg-pow: (-{a})^{b}
neg-paren-pow: -({a}^{b})
pow-neg-exp: {a}^-{b}
double-neg: --{a}
plus-pow: +{a}^{b}
pow-chain: {a}^{b}^{exp}
sub-pow: {num}-{a}^{b}
neg-times: -{a}*{num}
neg-lit-pow: -{small}^{b}
""",
    "percent": """
lit: {a}%
double: {a}%%
neg: -{a}%
pow-after: {a}%^{exp}
pow-before: {base}^{a}%
paren: ({a}+{num})%
times: {a}%*{num}
text: {text}%{sfx}
bool: {bool}%{sfx}
ref: {anycell}%
""",
    "concat": """
pair: {x}&{y}
triple: {x}&{y}&{scalar}
number-text: {trap}&{y}
after-add: {num}+{num}&{y}
before-cmp: {x}&{y}={scalar}
neg: -{num}&{y}
percent: {num}%&{y}
""",
    "compare": """
any: {scalar}{op}{scalar}
text-case: {text}{op}{text}
num-text: {n}{op}{text}
bool-num: {bool}{op}{n}
bool-text: {bool}{op}{text}
ref-blank: {cell}{op}{blankish}
chained: {scalar}{op}{scalar}{cmp}{scalar}
after-add: {num}+{num}{op}{num}
""",
    "coerce": """
text-num: {text}{op}{n}
num-text: {n}{op}{text}
bool-num: {bool}{op}{n}
text-bool: {text}{op}{bool}
ref-num: {cell}{op}{n}
neg-text: -{text}{sfx}
neg-bool: -{bool}{sfx}
pow-text: {text}^{half}
neg-ref: -{anycell}
""",
    "error": """
err-err: {e1}{arith}{e2}
err-concat: {e1}&{scalar}
concat-err: {scalar}&{e1}
div-zero: {num}/{zero}
cmp-err: {scalar}{cmp}{e1}
err-cmp: {e1}{cmp}{scalar}
div-then-err: {num}/0{arith}{e1}
neg: -{e1}{sfx}
percent: {e1}%{sfx}
pow-err: {e1}^{e2}
text-then-err: {text}+{e1}
ref-arith: {cell}{arith}{cell}
bare: {e1}
""",
    "refs": """
bare: {cell}
abs: Data!${c}${r}{sfx}
mixed-sum: {cell}+{cell}
times-one: {cell}*1
concat-empty: {cell}&""
beyond-data: Data!{beyond}{sfx}
""",
    # Every corner carries the sheet: an unprefixed corner would refer to Calc.
    "range": """
single-cell: Data!{one}:{one}
single-cell-arith: Data!{one}:{one}{arith}{n}
reversed-corners: Data!{hi}:{lo} {corner}
range-of-ranges: Data!{lo}:{lo}:Data!{hi} {corner}
ref-colon-ref: Data!{lo}:Data!{hi} {corner}
dollar-range: Data!{lo_abs}:{hi_abs} {corner}
""",
    "intersection": """
col-row: {col_of_a} {row_of_a}
corner: {rect} {from_corner}
disjoint: {rect} {before}
arith: ({col_of_a} {row_of_a}){arith}{num}
cross-sheet: {col_of_a} {other_row}
triple: {rect} {full_col} {full_row}
""",
    "union": """
bare: ({a},{b})
arith: ({a},{b}){arith}1
concat: ({a},{b})&""
cut-by-cell: ({left},{right}) {cut}
same-cell: ({a},{a})
nested: (({a},{b}),{c})
""",
    "whole-row-col": """
col-row: {q}{col}:{col} {q}{r}:{r}
abs-col-row: {q}${col}:${col} {q}${r}:${r}
col-cell: {q}{col}:{col} {q}{col}{r}
row-cell-arith: ({q}{r}:{r} {q}{col}{r}){arith}{n}
multi-col-row: {q}A:{last} {q}{col}{r}:{col}{r}
empty-far: {q}{far_col}:{far_col} {q}{far_row}:{far_row}
col-row-concat: ({q}{col}:{col} {q}{r}:{r})&""
""",
    "cross-sheet": """
bare: {c1}
arith: {c1}{arith}{c2}
concat: {c1}&{c2}
cmp: {c1}{cmp}{c2}
single-range: {q1}{one}:{one}{arith}{n}
""",
    # Same-sheet references to earlier probes only (acyclic), so recalculation order matters.
    "chain": """
seed: {cell}
plus-one: A{j}+1
abs: $A${j}*2
mixed: A${j}-$A{k}
concat: A{j}&A{k}
neg-pow: -A{j}^2
cmp: A{j}{cmp}A{k}
blank-calc: A{j}+B{j}
data-mix: A{j}{arith}{cell}
""",
    "mixed": """
tree: {tree}
""",
}


def _area(sheet: str, c0: int, r0: int, c1: int, r1: int) -> str:
    return f"{SHEETS[sheet]}{col_letter(c0)}{r0}:{col_letter(c1)}{r1}"


def _one_cell(g: _Gen) -> str:
    return f"{col_letter(g.rng.randrange(DATA_COLS))}{g.rng.randint(1, DATA_ROWS)}"


def _bind_range(g: _Gen) -> dict[str, str]:
    c0, r0, c1, r1 = g.rect()
    return {
        "one": _one_cell(g),
        "lo": f"{col_letter(c0)}{r0}",
        "hi": f"{col_letter(c1)}{r1}",
        "lo_abs": f"${col_letter(c0)}${r0}",
        "hi_abs": f"${col_letter(c1)}${r1}",
        "corner": f"Data!{col_letter(c0)}{r1}",  # one cell inside the rectangle
    }


def _bind_intersection(g: _Gen) -> dict[str, str]:
    a = g.rect()
    c, r = g.rng.randint(a[0], a[2]), g.rng.randint(a[1], a[3])
    # Ends above-left of `a`, so nothing is shared (#NULL!); Z99 when `a` touches column A or row 1.
    before = _area("Data", 0, 1, a[0] - 1, a[1] - 1) if a[0] > 0 and a[1] > 1 else "Data!Z99"
    return {
        "rect": _area("Data", *a),
        "col_of_a": _area("Data", c, a[1], c, a[3]),
        "row_of_a": _area("Data", a[0], r, a[2], r),
        # Starts at a's bottom-right corner, so it shares exactly that one cell with `a`.
        "from_corner": _area("Data", a[2], a[3], DATA_COLS - 1, DATA_ROWS),
        "before": before,
        "other_row": _area("My Data", a[0], r, a[2], r),
        "full_col": _area("Data", c, 1, c, DATA_ROWS),
        "full_row": _area("Data", 0, r, DATA_COLS - 1, r),
    }


def _bind_union(g: _Gen) -> dict[str, str]:
    c, r = g.rng.randrange(DATA_COLS), g.rng.randint(1, DATA_ROWS)
    other = (c + 2) % DATA_COLS
    return {
        "a": g.cell(dollars=False),
        "b": g.cell(dollars=False),
        "c": g.cell(dollars=False),
        # Two columns cut by one cell of the first: a single cell from a single area.
        "left": _area("Data", c, 1, c, DATA_ROWS),
        "right": _area("Data", other, 1, other, DATA_ROWS),
        "cut": f"Data!{col_letter(c)}{r}",
    }


def _bind_whole(g: _Gen) -> dict[str, str]:
    # A column and a row cross in one cell; the far pair is one blank cell outside the data.
    return {
        "q": SHEETS[g.pick(list(SHEETS))],
        "col": col_letter(g.rng.randrange(DATA_COLS)),
        "r": str(g.rng.randint(1, DATA_ROWS)),
        "last": col_letter(DATA_COLS - 1),
        "far_col": g.pick(["Z", "AA", "XFD"]),
        "far_row": str(g.rng.randint(100, 999)),
    }


def _bind_cross_sheet(g: _Gen) -> dict[str, str]:
    s1 = g.pick(list(SHEETS))
    return {"c1": g.cell(s1), "c2": g.cell(g.pick(list(SHEETS))), "q1": SHEETS[s1], "one": _one_cell(g)}


def _bind_chain(g: _Gen, i: int) -> dict[str, str]:
    if i == 1:
        return {}
    return {"j": str(g.rng.randint(max(1, i - 10), i - 1)), "k": str(g.rng.randint(1, i - 1))}


BINDERS: dict[str, Callable[[_Gen, int], dict[str, str]]] = {
    "literal": lambda g, _: {"a": g.pick(LITERALS)},
    "unary-power": lambda g, _: {"a": g.number(), "b": g.pick(["0", "1", "2", "3", "0.5"])},
    "percent": lambda g, _: {"a": g.number()},
    "concat": lambda g, _: {"x": g.scalar(), "y": g.scalar()},
    "compare": lambda g, _: {"op": g.pick(CMP_OPS)},
    "coerce": lambda g, _: {"op": g.pick(ARITH)},
    "error": lambda g, _: {"e1": g.pick(ERRORS), "e2": g.pick(ERRORS)},
    "refs": lambda g, _: {"c": col_letter(g.rng.randrange(DATA_COLS)), "r": str(g.rng.randint(1, DATA_ROWS))},
    "range": lambda g, _: _bind_range(g),
    "intersection": lambda g, _: _bind_intersection(g),
    "union": lambda g, _: _bind_union(g),
    "whole-row-col": lambda g, _: _bind_whole(g),
    "cross-sheet": lambda g, _: _bind_cross_sheet(g),
    "chain": _bind_chain,
    "mixed": lambda g, _: {"tree": g.expr()},
}

CATEGORIES = list(TEMPLATES)
_PLACEHOLDER = re.compile(r"\{(\w+)\}")


def _forms(category: str, i: int) -> dict[str, str]:
    forms = dict(line.split(": ", 1) for line in TEMPLATES[category].strip().splitlines())
    if category == "chain":  # the first chain probe has no earlier probe to reference
        return {"seed": forms["seed"]} if i == 1 else {k: v for k, v in forms.items() if k != "seed"}
    return forms


def _fill(g: _Gen, template: str, bound: dict[str, str]) -> str:
    return _PLACEHOLDER.sub(lambda m: bound[m[1]] if m[1] in bound else DRAWS[m[1]](g), template)


def _probe(g: _Gen, category: str, i: int, seen: set[str]) -> tuple[str, str]:
    """(formula, form) for probe `i`, redrawn while it repeats a formula already in the case."""
    forms = _forms(category, i)
    formula, form = "", ""
    for _ in range(REDRAWS):
        bound = BINDERS[category](g, i)
        form = g.pick(list(forms))
        formula = "=" + _fill(g, forms[form], bound)
        if formula not in seen:
            break
    seen.add(formula)
    return formula, form


def _data_sheet(rng: random.Random) -> dict[str, Any]:
    cells: dict[str, Any] = {}
    for r in range(1, DATA_ROWS + 1):
        for c in range(DATA_COLS):
            v = rng.choice(DATA_POOL)
            if v is not None:
                cells[f"{col_letter(c)}{r}"] = v
    return cells


def generate_operators_refs(seed: int, per_category: int = PER_CATEGORY) -> list[dict[str, Any]]:
    """Return one `rustsheet-case/v1` spec per category, `per_category` tagged probes each."""
    cases: list[dict[str, Any]] = []
    for category in CATEGORIES:
        g = _Gen(random.Random(f"{seed}/{category}"))
        calc: dict[str, str] = {}
        hypotheses: dict[str, str] = {}
        seen: set[str] = set()
        for i in range(1, per_category + 1):
            calc[f"A{i}"], form = _probe(g, category, i, seen)
            hypotheses[f"Calc!A{i}"] = f"{category}/{form}"
        cases.append(
            {
                "format": "rustsheet-case/v1",
                "description": f"operators-refs probes ({category}) seed={seed}; M1 operators and references only",
                "tags": ["generated", "M1", "operators-refs", category],
                "sheets": [
                    *({"name": name, "cells": _data_sheet(g.rng)} for name in SHEETS),
                    {"name": "Calc", "cells": calc},
                ],
                "expected": {},
                "check": [f"Calc!{c}" for c in calc],
                "hypotheses": hypotheses,
            }
        )
    return cases
