"""Synthetic workbook generator (phase P0: type-directed, catalog-driven, seeded).

Produces `rustsheet-case/v1` specs with a realistic data sheet and a calc sheet of
generated formulas. Expectations are left empty: run `rustsheet-oracle fill` to get
Excel's answers, then `rustsheet-eval run` to diff.

Design: docs/architecture/synthetic-generator.md. Later phases add adaptive (bandit)
weighting toward failing functions, metamorphic relations, and test-case reduction.
"""

from __future__ import annotations

import hashlib
import json
import random
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml

MILESTONES = ["M0", "M1", "M2", "M3", "M4", "M5", "M6", "M7", "M8", "M9", "M10"]

# Edge-value pools per type (P0 subset of the research design).
NUMBER_POOL: list[float] = [
    0,
    1,
    -1,
    2,
    0.5,
    -0.5,
    0.1,
    0.2,
    0.3,
    1e-15,
    1e15,
    123456789.123,
    -0.0001,
    3.14159,
    2.5,
    -2.5,
    1.005,
    100,
    42,
    7,
]
TEXT_POOL: list[str] = [
    "",
    "a",
    "abc",
    "ABC",
    "Hello World",
    " x ",
    "1",
    "1.5",
    "TRUE",
    "#N/A",
    "ü",
    "a*",
    "?b",
    "x~*",
]
CRITERIA_POOL: list[Any] = [">0", "<5", ">=2", "<>1", "=a", "a*", "*c", "?b?", 1, "abc", "", "<>"]
# LET variables and LAMBDA parameters: none is a cell reference (`x1` is one), a function name (`N`) or `r`/`c`.
NAME_POOL: list[str] = ["x", "y", "z", "w", "v", "acc"]
# Functions passed by name as a LAMBDA (eta-reduced, `BYROW(A1:B3,SUM)`); Excel stores them `_xleta.SUM`
# and a bare `SUM` read from a file is #NAME? (docs/excel-semantics/xlsx-storage.md). They take 1 to 255
# arguments, so any arity fits (probes/m7-argument-modes: `MAKEARRAY(2,2,SUM)` is SUM(row,col)).
ETA_POOL: list[str] = ["SUM", "MAX", "MIN", "AVERAGE", "COUNT", "PRODUCT"]
MASK_OPS: list[str] = [">", "<", "=", "<>"]


def col_letter(index: int) -> str:
    """Column letter for a 0-based index within A..Z (generated data sheets are narrow)."""
    return chr(ord("A") + index)


def load_excel_functions(path: Path) -> dict[str, dict[str, str]]:
    """Rows of eval/catalog/excel-functions.tsv (Microsoft's en-US function list, T-0040) by name.

    `#` lines are provenance comments; the first other line is the header. Duplicate names raise,
    so the source list itself cannot list a function twice."""
    lines = [ln for ln in path.read_text(encoding="utf-8").splitlines() if ln and not ln.startswith("#")]
    header = lines[0].split("\t")
    rows: dict[str, dict[str, str]] = {}
    for ln in lines[1:]:
        row = dict(zip(header, ln.split("\t"), strict=True))
        if row["name"] in rows:
            msg = f"{row['name']} is listed twice in {path.name}"
            raise ValueError(msg)
        rows[row["name"]] = row
    return rows


@dataclass
class Catalog:
    operators: list[dict[str, Any]]
    functions: list[dict[str, Any]]

    @staticmethod
    def load(path: Path) -> Catalog:
        data = yaml.safe_load(path.read_text())
        return Catalog(operators=data.get("operators", []), functions=data.get("functions", []))

    def scoped(self, milestone: str) -> Catalog:
        """Operators and generator-ready functions in scope at `milestone`.

        Catalog entries without `params` only name a function (T-0040: every Excel function is
        listed, most without a signature yet), so the generator cannot call them. Volatile functions
        (RAND) have a signature but are left out: their value never matches an oracle's."""
        limit = MILESTONES.index(milestone)

        def ok(entry: dict[str, Any]) -> bool:
            return MILESTONES.index(entry.get("milestone", "M10")) <= limit

        functions = [f for f in self.functions if "params" in f and not f.get("volatile") and ok(f)]
        return Catalog([o for o in self.operators if ok(o)], functions)


@dataclass
class Gen:
    rng: random.Random
    catalog: Catalog
    data_rows: int
    data_cols: int
    max_depth: int
    used: set[str] = field(default_factory=set)
    # LET variables and LAMBDA parameters an expression here may use; `pending` are declared but not yet
    # visible (a LET variable is visible after its value, a LAMBDA parameter in the body).
    scope: list[str] = field(default_factory=list)
    pending: list[str] = field(default_factory=list)

    # -- references into the data sheet --------------------------------------------------
    def cell_ref(self) -> str:
        col = col_letter(self.rng.randrange(self.data_cols))
        row = self.rng.randrange(1, self.data_rows + 1)
        dollar_c = "$" if self.rng.random() < 0.2 else ""
        dollar_r = "$" if self.rng.random() < 0.2 else ""
        return f"Data!{dollar_c}{col}{dollar_r}{row}"

    def range_ref(self, width: int = 1, *, full: bool = False) -> str:
        """A `width`-column range of data rows; `full` spans every row, so two full ranges are the same height."""
        c0 = self.rng.randrange(self.data_cols - width + 1)
        r0, r1 = 1, self.data_rows
        if not full:
            r0 = self.rng.randrange(1, self.data_rows)
            r1 = self.rng.randrange(r0, self.data_rows + 1)
        a = col_letter(c0)
        b = col_letter(c0 + width - 1)
        return f"Data!{a}{r0}:{b}{r1}"

    # -- literals --------------------------------------------------------------------------
    def literal(self, typ: str) -> str:
        if typ == "number" or (typ == "any" and self.rng.random() < 0.5):
            v = self.rng.choice(NUMBER_POOL)
            return repr(v) if v >= 0 else f"({v!r})"
        if typ == "bool":
            return self.rng.choice(["TRUE", "FALSE"])
        s = self.rng.choice(TEXT_POOL)
        return '"' + s.replace('"', '""') + '"'

    # -- expressions -----------------------------------------------------------------------
    def expr(self, typ: str, depth: int) -> str:
        roll = self.rng.random()
        if depth >= self.max_depth or roll < 0.25:
            if self.scope and self.rng.random() < 0.5:
                return self.rng.choice(self.scope)
            return self.literal(typ) if self.rng.random() < 0.4 else self.cell_ref()
        if roll < 0.55 and self.catalog.operators:
            return self.operator(typ, depth)
        if self.catalog.functions:
            return self.call(typ, depth)
        return self.literal(typ)

    def operator(self, typ: str, depth: int) -> str:
        ops = [o for o in self.catalog.operators if typ in ("any", o["returns"])] or self.catalog.operators
        op = self.weighted(ops)
        self.used.add(op["op"])
        args = [self.expr(t, depth + 1) for t in op["args"]]
        if op["op"] == "neg":
            return f"-{args[0]}"
        if op["op"] == "%":
            return f"{args[0]}%"
        return f"({args[0]}{op['op']}{args[1]})"

    def call(self, typ: str, depth: int) -> str:
        fns = [f for f in self.catalog.functions if typ in ("any", f["returns"]) or f["returns"] == "any"]
        fn = self.weighted(fns or self.catalog.functions)
        self.used.add(fn["name"])
        args: list[str] = []
        params: list[dict[str, Any]] = fn["params"]
        outer, outer_pending = len(self.scope), self.pending
        self.pending = []
        declared = 0  # names this call declares (LET variables, LAMBDA parameters)
        repeated = 1  # how many arguments the last repeating param got (MAP's arrays fix its LAMBDA's arity)
        i = 0
        while i < len(params):
            p = params[i]
            # Excel allows omitting trailing optionals only.
            # `always` optionals are never omitted (CELL's reference: without it CELL reads the cell last changed).
            if (
                p.get("optional")
                and not p.get("always")
                and self.rng.random() < 0.4
                and all(q.get("optional") for q in params[i:])
            ):
                break
            rep = p.get("repeat")
            if isinstance(rep, dict):
                # Pair-repeat (SUMIFS criteria_range/criteria): the whole group repeats together.
                members = [q for q in params if q["name"] in rep["group"]]
                for _ in range(self.rng.randint(rep.get("min", 1), min(rep.get("max", 3), 3))):
                    args.extend(self.argument(q, depth) for q in members)
                i += len(members)
                continue
            count = self.rng.randint(1, 3) if p.get("repeat") else 1
            if p.get("repeat"):
                repeated = count
            if p.get("body"):
                # LAMBDA's calculation sees all its parameters.
                self.scope += self.pending
                self.pending = []
            args.extend(self.argument(p, depth, arity=repeated) for _ in range(count))
            if p.get("mode") == "name":
                declared += count
            i += 1
        del self.scope[outer:]
        self.pending = outer_pending
        invoked = ""
        if fn.get("invoke"):
            # A LAMBDA left uncalled is #CALC!, so the generator calls it with one value per parameter.
            invoked = "(" + ",".join(self.expr("any", depth + 1) for _ in range(declared)) + ")"
        return f"{fn.get('spelled', fn['name'])}({','.join(args)}){invoked}"

    def fresh_name(self) -> str:
        taken = set(self.scope) | set(self.pending)
        free = [n for n in NAME_POOL if n not in taken]
        return self.rng.choice(free) if free else f"v_{len(taken)}"

    def lambda_arg(self, arity: int, depth: int) -> str:
        """A LAMBDA of `arity` parameters, or (30%) a function passed by name."""
        if self.rng.random() < 0.3:
            return "_xleta." + self.rng.choice(ETA_POOL)
        outer = len(self.scope)
        names = []
        for _ in range(arity):
            names.append(self.fresh_name())
            self.scope.append(names[-1])
        body = self.expr("any", depth + 1)
        del self.scope[outer:]
        return f"LAMBDA({','.join(names)},{body})"

    def argument(self, p: dict[str, Any], depth: int, arity: int = 1) -> str:
        mode = p.get("mode", "value")
        if mode == "name":
            name = self.fresh_name()
            self.pending.append(name)
            return name
        if mode == "lambda":
            n = p.get("arity", 1)
            return self.lambda_arg(arity if n == "repeat" else int(n), depth)
        if mode == "array" and p.get("mask"):
            # FILTER's include: one TRUE/FALSE per data row, so it fits a `full` array.
            text = f"({self.range_ref(full=True)}{self.rng.choice(MASK_OPS)}{self.literal('number')})"
        elif mode in ("array", "aggregate", "aggregate_a") and self.scope and self.rng.random() < 0.5:
            # A LAMBDA parameter or LET variable often holds the row, column or array (BYROW's SUM(x)).
            text = self.rng.choice(self.scope)
        else:
            text = self.argument_text(p, mode, depth)
        # A LET variable is visible from the argument after its value on.
        self.scope += self.pending
        self.pending = []
        return text

    def argument_text(self, p: dict[str, Any], mode: str, depth: int) -> str:
        if mode == "range" and p.get("cell"):
            # ISFORMULA lifts over a range; one cell keeps the result from spilling into the next formula.
            return self.cell_ref()
        if p.get("vector") and (mode == "range" or self.rng.random() < 0.5):
            # LOOKUP/XLOOKUP/XMATCH search one column here and WRAPROWS/WRAPCOLS wrap one; 2-D only gives an error.
            return self.range_ref(width=1, full=p.get("full", False))
        # An `array` vector that is not a range is an expression. `full` arrays (GROUPBY, SORTBY, FILTER) are
        # always ranges of every row: their heights must agree.
        if not p.get("vector") and (
            mode == "range" or (mode == "array" and (p.get("full") or self.rng.random() < 0.5))
        ):
            return self.range_ref(
                width=self.rng.randint(1, min(4, self.data_cols)), full=p.get("full", False)
            )
        if mode == "criteria":
            c = self.rng.choice(CRITERIA_POOL)
            return repr(c) if isinstance(c, int) else '"' + str(c) + '"'
        if mode in ("aggregate", "aggregate_a") and self.rng.random() < 0.5:
            return self.range_ref()
        in_domain = self.domain_value(p.get("domain") or {})
        return in_domain if in_domain is not None else self.value_expr(p, depth)

    def value_expr(self, p: dict[str, Any], depth: int) -> str:
        """A free expression for a `value` param. A `lift: arrays` param refuses a multi-cell range
        (SPEC-0011 `LiftArrays`), which a LAMBDA parameter or LET variable may hold (BYROW's row),
        so its expression sees no names (T-1776)."""
        if p.get("lift") != "arrays":
            return self.expr(p.get("type", "any"), depth + 1)
        outer, self.scope = self.scope, []
        try:
            return self.expr(p.get("type", "any"), depth + 1)
        finally:
            self.scope = outer

    def domain_value(self, dom: dict[str, Any]) -> str | None:
        """An in-domain literal, or None to draw a free expression (a soft domain, 30% of the time)."""
        # A `hard` domain is never violated: outside it Excel refuses the workbook (MUNIT(10^6)).
        if not (dom.get("int") or dom.get("enum")) or not (dom.get("hard") or self.rng.random() < 0.7):
            return None
        if dom.get("int"):
            return str(self.rng.randint(dom.get("min", 0), dom.get("max", 10)))
        v = self.rng.choice(dom["enum"])
        # A number enum (a bond's frequency 1, 2 or 4) is a number literal; text would be coerced, not the same call.
        return str(v) if isinstance(v, int | float) and not isinstance(v, bool) else '"' + str(v) + '"'

    def weighted(self, items: list[dict[str, Any]]) -> dict[str, Any]:
        # sqrt-tempered prior so rare functions still get coverage
        weights = [float(i.get("prior", 1)) ** 0.5 for i in items]
        return self.rng.choices(items, weights=weights, k=1)[0]

    # -- data sheet ------------------------------------------------------------------------
    def data_value(self) -> Any:
        r = self.rng.random()
        if r < 0.55:
            return self.rng.choice(NUMBER_POOL)
        if r < 0.75:
            return self.rng.choice(TEXT_POOL) or "x"
        if r < 0.85:
            return self.rng.choice([True, False])
        return None  # blank


def generate_case(
    catalog: Catalog,
    seed: int,
    *,
    milestone: str,
    formulas: int = 40,
    data_rows: int = 12,
    data_cols: int = 5,
    max_depth: int = 3,
) -> dict[str, Any]:
    rng = random.Random(seed)
    g = Gen(rng, catalog.scoped(milestone), data_rows, data_cols, max_depth)
    data: dict[str, Any] = {}
    for r in range(1, data_rows + 1):
        for c in range(data_cols):
            v = g.data_value()
            if v is not None:
                if isinstance(v, str) and v.startswith("="):
                    v = {"t": "s", "v": v}
                data[f"{chr(ord('A') + c)}{r}"] = v
    calc: dict[str, Any] = {}
    for i in range(1, formulas + 1):
        typ = rng.choice(["number", "number", "any", "text", "bool"])
        calc[f"A{i}"] = "=" + g.expr(typ, 0)
    return {
        "format": "rustsheet-case/v1",
        "description": f"generated P0 seed={seed} milestone={milestone}",
        "tags": ["generated", milestone, *sorted(g.used)],
        "sheets": [{"name": "Data", "cells": data}, {"name": "Calc", "cells": calc}],
        "expected": {},
        "check": [f"Calc!A{i}" for i in range(1, formulas + 1)],
    }


def write_case(spec: dict[str, Any], out_dir: Path) -> Path:
    body = json.dumps(spec, indent=1, sort_keys=True) + "\n"
    digest = hashlib.sha256(body.encode()).hexdigest()[:16]
    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / f"{digest}.case.json"
    path.write_text(body)
    return path
