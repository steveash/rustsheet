"""Policy `operators-refs`: operator and reference probes for M1 (suite `gen-m1`, T-0117).

Only literals, operators and references, never a function name, so the suite measures M1 scope
(docs/milestones.md): unary minus vs `^`, `%`, `&`, comparisons across types, text/number/bool
coercion, error propagation, the range/union/intersection operators, `$`/mixed refs, whole rows
and columns, cross-sheet and quoted sheet names, and recalculation chains.

One case per category; every probe is one cell on sheet `Calc`, tagged `category/form` in the
case's `hypotheses` map. Every probe yields one value: a multi-cell result would spill into the
next probe (spec formulas are dynamic-array formulas, decision 0007), so ranges reach a probe only
through an intersection or a single-cell range. Whole rows and columns appear only as intersection
operands, which never build million-cell work. No volatile or locale-dependent constructs: no date
or currency text, no thousands separators. Expectations stay empty until `rustsheet-oracle fill`
(T-0109).
"""

from __future__ import annotations

import random
from collections.abc import Callable
from typing import Any

PER_CATEGORY = 250
DATA_ROWS = 12
DATA_COLS = 5
# Quoted names: a space, and an apostrophe (doubled inside the quotes).
SHEETS = {"Data": "Data!", "My Data": "'My Data'!", "Q'1": "'Q''1'!"}

NUMBERS = ["0", "1", "2", "3", "0.5", "1.5", "10", "100", "0.1", "1E+3", "2.5", "7"]
TEXTS = ['"a"', '"A"', '""', '"abc"', '"1"', '" 2 "', '"1e3"', '"0.5"', '"-1"', '"TRUE"', '"x y"']
BOOLS = ["TRUE", "FALSE"]
EMPTY = '""'  # the empty-text literal
ZERO_TEXT = '"0"'
ERRORS = ["#DIV/0!", "#N/A", "#VALUE!", "#NUM!", "#REF!", "#NAME?", "#NULL!"]
CMP = ["=", "<>", "<", ">", "<=", ">="]
ARITH = ["+", "-", "*", "/"]
# Values seeded into the data sheets: numbers, text (numeric-looking too), bools, errors, blanks.
DATA_POOL: list[Any] = [
    0,
    1,
    -1,
    2,
    0.5,
    3.25,
    100,
    1e-15,
    "a",
    "ABC",
    "1",
    " 2 ",
    "",
    True,
    False,
    {"t": "e", "v": "#N/A"},
    {"t": "e", "v": "#DIV/0!"},
    None,
    None,
]

Probe = tuple[str, str]


def _col(c: int) -> str:
    return chr(ord("A") + c)


class _Gen:
    def __init__(self, rng: random.Random) -> None:
        self.rng = rng

    def pick(self, items: list[str]) -> str:
        return self.rng.choice(items)

    def cell(self, sheet: str = "Data", dollars: bool = True) -> str:
        c, r = self.rng.randrange(DATA_COLS), self.rng.randint(1, DATA_ROWS)
        dc = "$" if dollars and self.rng.random() < 0.3 else ""
        dr = "$" if dollars and self.rng.random() < 0.3 else ""
        return f"{SHEETS[sheet]}{dc}{_col(c)}{dr}{r}"

    def any_sheet_cell(self) -> str:
        return self.cell(self.pick(list(SHEETS)))

    def scalar(self) -> str:
        """Any operand: literal of any type or a data cell."""
        roll = self.rng.random()
        if roll < 0.3:
            return self.pick(NUMBERS)
        if roll < 0.45:
            return self.pick(TEXTS)
        if roll < 0.55:
            return self.pick(BOOLS)
        if roll < 0.62:
            return self.pick(ERRORS)
        return self.any_sheet_cell()

    def number(self) -> str:
        return self.pick(NUMBERS) if self.rng.random() < 0.6 else self.cell()

    def rect(self) -> tuple[int, int, int, int]:
        c0, c1 = sorted(self.rng.sample(range(DATA_COLS), 2))
        r0, r1 = sorted(self.rng.sample(range(1, DATA_ROWS + 1), 2))
        return c0, r0, c1, r1


def _area(sheet: str, c0: int, r0: int, c1: int, r1: int) -> str:
    return f"{SHEETS[sheet]}{_col(c0)}{r0}:{_col(c1)}{r1}"


def _unary_power(g: _Gen) -> Probe:
    a, b = g.number(), g.pick(["0", "1", "2", "3", "0.5"])
    forms = {
        "neg-pow": f"-{a}^{b}",
        "paren-neg-pow": f"(-{a})^{b}",
        "neg-paren-pow": f"-({a}^{b})",
        "pow-neg-exp": f"{a}^-{b}",
        "double-neg": f"--{a}",
        "plus-pow": f"+{a}^{b}",
        "pow-chain": f"{a}^{b}^{g.pick(['0', '1', '2'])}",
        "sub-pow": f"{g.number()}-{a}^{b}",
        "neg-times": f"-{a}*{g.number()}",
        "neg-lit-pow": f"-{g.pick(['2', '3', '0.5'])}^{b}",
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _percent(g: _Gen) -> Probe:
    a, b = g.number(), g.number()
    forms = {
        "lit": f"{a}%",
        "double": f"{a}%%",
        "neg": f"-{a}%",
        "pow-after": f"{a}%^2",
        "pow-before": f"2^{a}%",
        "paren": f"({a}+{b})%",
        "times": f"{a}%*{b}",
        "text": f"{g.pick(TEXTS)}%",
        "bool": f"{g.pick(BOOLS)}%",
        "ref": f"{g.any_sheet_cell()}%",
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _concat(g: _Gen) -> Probe:
    x, y = g.scalar(), g.scalar()
    forms = {
        "pair": f"{x}&{y}",
        "triple": f"{x}&{y}&{g.scalar()}",
        "number-text": f"{g.pick(['0.1+0.2', '1/3', '1E+20', '-0', '2/3*3', '1E-5'])}&{y}",
        "after-add": f"{g.number()}+{g.number()}&{y}",
        "before-cmp": f"{x}&{y}={g.scalar()}",
        "neg": f"-{g.number()}&{y}",
        "percent": f"{g.number()}%&{y}",
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _compare(g: _Gen) -> Probe:
    op = g.pick(CMP)
    forms = {
        "any": f"{g.scalar()}{op}{g.scalar()}",
        "text-case": f"{g.pick(TEXTS)}{op}{g.pick(TEXTS)}",
        "num-text": f"{g.pick(NUMBERS)}{op}{g.pick(TEXTS)}",
        "bool-num": f"{g.pick(BOOLS)}{op}{g.pick(NUMBERS)}",
        "bool-text": f"{g.pick(BOOLS)}{op}{g.pick(TEXTS)}",
        "ref-blank": f"{g.cell()}{op}{g.pick(['0', EMPTY, 'FALSE'])}",
        "chained": f"{g.scalar()}{op}{g.scalar()}{g.pick(CMP)}{g.scalar()}",
        "after-add": f"{g.number()}+{g.number()}{op}{g.number()}",
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _coerce(g: _Gen) -> Probe:
    op = g.pick(ARITH)
    forms = {
        "text-num": f"{g.pick(TEXTS)}{op}{g.pick(NUMBERS)}",
        "num-text": f"{g.pick(NUMBERS)}{op}{g.pick(TEXTS)}",
        "bool-num": f"{g.pick(BOOLS)}{op}{g.pick(NUMBERS)}",
        "text-bool": f"{g.pick(TEXTS)}{op}{g.pick(BOOLS)}",
        "ref-num": f"{g.cell()}{op}{g.pick(NUMBERS)}",
        "neg-text": f"-{g.pick(TEXTS)}",
        "neg-bool": f"-{g.pick(BOOLS)}",
        "pow-text": f"{g.pick(TEXTS)}^{g.pick(['2', '0.5'])}",
        "neg-ref": f"-{g.any_sheet_cell()}",
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _error(g: _Gen) -> Probe:
    e1, e2 = g.pick(ERRORS), g.pick(ERRORS)
    forms = {
        "err-err": f"{e1}{g.pick(ARITH)}{e2}",
        "err-concat": f"{e1}&{g.scalar()}",
        "concat-err": f"{g.scalar()}&{e1}",
        "div-zero": f"{g.number()}/{g.pick(['0', ZERO_TEXT, 'FALSE', g.cell()])}",
        "cmp-err": f"{g.scalar()}{g.pick(CMP)}{e1}",
        "err-cmp": f"{e1}{g.pick(CMP)}{g.scalar()}",
        "div-then-err": f"1/0+{e1}",
        "neg": f"-{e1}",
        "percent": f"{e1}%",
        "pow-err": f"{e1}^{e2}",
        "text-then-err": f'"a"+{e1}',
        "ref-arith": f"{g.cell()}{g.pick(ARITH)}{g.cell()}",
        "bare": e1,
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _refs(g: _Gen) -> Probe:
    forms = {
        "bare": g.cell(),
        "abs": f"Data!${_col(g.rng.randrange(DATA_COLS))}${g.rng.randint(1, DATA_ROWS)}",
        "mixed-sum": f"{g.cell()}+{g.cell()}",
        "times-one": f"{g.cell()}*1",
        "concat-empty": f'{g.cell()}&""',
        "beyond-data": f"Data!{g.pick(['$Z$99', 'F1', 'A13', 'XFD1048576'])}{g.pick(['', '+1', '&' + EMPTY])}",
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _range(g: _Gen) -> Probe:
    c, r = g.rng.randrange(DATA_COLS), g.rng.randint(1, DATA_ROWS)
    one = f"{_col(c)}{r}"
    c0, r0, c1, r1 = g.rect()
    forms = {
        "single-cell": f"Data!{one}:{one}",
        "single-cell-arith": f"Data!{one}:{one}{g.pick(ARITH)}{g.pick(NUMBERS)}",
        "reversed-corners": f"Data!{_col(c1)}{r1}:{_col(c0)}{r0} Data!{_col(c0)}{r0}",
        "range-of-ranges": f"Data!{_col(c0)}{r0}:{_col(c0)}{r0}:{_col(c1)}{r1} Data!{_col(c1)}{r1}",
        "ref-colon-ref": f"Data!{_col(c0)}{r0}:Data!{_col(c1)}{r1} Data!{_col(c0)}{r1}",
        "dollar-range": f"Data!${_col(c0)}${r0}:${_col(c1)}${r1} Data!{_col(c1)}{r0}",
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _intersection(g: _Gen) -> Probe:
    a = g.rect()
    b = g.rect()
    c = g.rng.randint(a[0], a[2])
    r = g.rng.randint(a[1], a[3])
    forms = {
        # One column of `a` against one row of `a`: always the single cell (c, r).
        "col-row": f"{_area('Data', c, a[1], c, a[3])} {_area('Data', a[0], r, a[2], r)}",
        # Two rectangles: one cell, several (spill, rare) or none (#NULL!); keep only 1 or 0.
        "rect-rect": f"{_area('Data', *a)} {_area('Data', *b)}",
        "disjoint": "Data!A1:B2 Data!D5:E6",
        "arith": f"({_area('Data', c, a[1], c, a[3])} {_area('Data', a[0], r, a[2], r)}){g.pick(ARITH)}{g.number()}",
        "cross-sheet": f"{_area('Data', *a)} {_area('My Data', *a)}",
        "triple": f"{_area('Data', *a)} {_area('Data', c, 1, c, DATA_ROWS)} {_area('Data', 0, r, DATA_COLS - 1, r)}",
    }
    form = g.pick(list(forms))
    if form == "rect-rect":
        w = min(a[2], b[2]) - max(a[0], b[0]) + 1
        h = min(a[3], b[3]) - max(a[1], b[1]) + 1
        if w > 0 and h > 0 and w * h > 1:
            return _intersection(g)  # would spill
    return f"={forms[form]}", form


def _union(g: _Gen) -> Probe:
    a, b = g.cell(dollars=False), g.cell(dollars=False)
    c, r = g.rng.randrange(DATA_COLS), g.rng.randint(1, DATA_ROWS)
    forms = {
        "bare": f"({a},{b})",
        "arith": f"({a},{b}){g.pick(ARITH)}1",
        "concat": f'({a},{b})&""',
        # Union of two columns cut by one row picks one cell from one area (single result).
        "cut-by-row": f"(Data!{_col(c)}1:{_col(c)}{DATA_ROWS},Data!{_col((c + 2) % DATA_COLS)}1:{_col((c + 2) % DATA_COLS)}{DATA_ROWS}) Data!{_col(c)}{r}",
        "same-cell": f"({a},{a})",
        "nested": f"(({a},{b}),{g.cell(dollars=False)})",
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _whole_row_col(g: _Gen) -> Probe:
    c, r = g.rng.randrange(DATA_COLS), g.rng.randint(1, DATA_ROWS)
    col, sheet = _col(c), g.pick(list(SHEETS))
    q = SHEETS[sheet]
    forms = {
        "col-row": f"{q}{col}:{col} {q}{r}:{r}",
        "abs-col-row": f"{q}${col}:${col} {q}${r}:${r}",
        "col-cell": f"{q}{col}:{col} {q}{col}{r}",
        "row-cell-arith": f"({q}{r}:{r} {q}{col}{r}){g.pick(ARITH)}{g.pick(NUMBERS)}",
        "multi-col-row": f"{q}A:{_col(DATA_COLS - 1)} {q}{col}{r}:{col}{r}",
        "empty-far": f"{q}Z:Z {q}100:100",
        "col-row-concat": f'({q}{col}:{col} {q}{r}:{r})&""',
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _cross_sheet(g: _Gen) -> Probe:
    s1, s2 = g.pick(list(SHEETS)), g.pick(list(SHEETS))
    forms = {
        "bare": g.cell(s1),
        "arith": f"{g.cell(s1)}{g.pick(ARITH)}{g.cell(s2)}",
        "concat": f"{g.cell(s1)}&{g.cell(s2)}",
        "cmp": f"{g.cell(s1)}{g.pick(CMP)}{g.cell(s2)}",
        "single-range": f"{SHEETS[s1]}B2:B2*2",
    }
    form = g.pick(list(forms))
    return f"={forms[form]}", form


def _chain_factory() -> Callable[[_Gen], Probe]:
    """Same-sheet references to earlier probes only (acyclic), so recalculation order matters."""
    count = [0]

    def chain(g: _Gen) -> Probe:
        count[0] += 1
        i = count[0]
        if i == 1:
            return f"={g.cell()}", "seed"
        j = g.rng.randint(max(1, i - 10), i - 1)
        k = g.rng.randint(1, i - 1)
        forms = {
            "plus-one": f"A{j}+1",
            "abs": f"$A${j}*2",
            "mixed": f"A${j}-$A{k}",
            "concat": f"A{j}&A{k}",
            "neg-pow": f"-A{j}^2",
            "cmp": f"A{j}{g.pick(CMP)}A{k}",
            "blank-calc": f"A{j}+B{j}",
            "data-mix": f"A{j}{g.pick(ARITH)}{g.cell()}",
        }
        form = g.pick(list(forms))
        return f"={forms[form]}", form

    return chain


def _mixed(g: _Gen) -> Probe:
    def expr(depth: int) -> str:
        roll = g.rng.random()
        if depth >= 3 or roll < 0.35:
            return g.scalar()
        if roll < 0.45:
            return f"-{expr(depth + 1)}"
        if roll < 0.5:
            return f"{expr(depth + 1)}%"
        op = g.pick([*ARITH, "^", "&", *CMP])
        return f"({expr(depth + 1)}{op}{expr(depth + 1)})"

    return f"={expr(0)}", "tree"


def _data_sheet(rng: random.Random) -> dict[str, Any]:
    cells: dict[str, Any] = {}
    for r in range(1, DATA_ROWS + 1):
        for c in range(DATA_COLS):
            v = rng.choice(DATA_POOL)
            if v is not None:
                cells[f"{_col(c)}{r}"] = v
    return cells


CATEGORIES: dict[str, Callable[[], Callable[[_Gen], Probe]]] = {
    "unary-power": lambda: _unary_power,
    "percent": lambda: _percent,
    "concat": lambda: _concat,
    "compare": lambda: _compare,
    "coerce": lambda: _coerce,
    "error": lambda: _error,
    "refs": lambda: _refs,
    "range": lambda: _range,
    "intersection": lambda: _intersection,
    "union": lambda: _union,
    "whole-row-col": lambda: _whole_row_col,
    "cross-sheet": lambda: _cross_sheet,
    "chain": _chain_factory,
    "mixed": lambda: _mixed,
}


def generate_operators_refs(seed: int, per_category: int = PER_CATEGORY) -> list[dict[str, Any]]:
    """Return one `rustsheet-case/v1` spec per category, `per_category` tagged probes each."""
    cases: list[dict[str, Any]] = []
    for category, factory in CATEGORIES.items():
        rng = random.Random(f"{seed}/{category}")
        g = _Gen(rng)
        make = factory()
        probes = [make(g) for _ in range(per_category)]
        calc = {f"A{i}": f for i, (f, _) in enumerate(probes, 1)}
        cases.append(
            {
                "format": "rustsheet-case/v1",
                "description": f"operators-refs probes ({category}) seed={seed}; M1 operators and references only",
                "tags": ["generated", "M1", "operators-refs", category],
                "sheets": [
                    *({"name": name, "cells": _data_sheet(rng)} for name in SHEETS),
                    {"name": "Calc", "cells": calc},
                ],
                "expected": {},
                "check": [f"Calc!{c}" for c in calc],
                "hypotheses": {f"Calc!A{i}": f"{category}/{t}" for i, (_, t) in enumerate(probes, 1)},
            }
        )
    return cases
