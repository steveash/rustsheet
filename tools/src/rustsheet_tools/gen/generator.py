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
import re
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml

from rustsheet_tools.gen import database
from rustsheet_tools.gen.common import quoted
from rustsheet_tools.gen.pivots import PivotSource, build_source, pivot_call
from rustsheet_tools.gen.pools import POOLS, lambda_arity

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
# Criteria as text; the text-number (`"1"`, `"=2"`, `"1E1"`), tilde-escape (`"x~*"`, `"~?"`; `"x~*"` is also a TEXT_POOL
# data value) and boolean entries exercise Excel's text-to-number criteria matching (T-2717). Ints are emitted as numeric literals.
CRITERIA_POOL: list[Any] = [
    ">0", "<5", ">=2", "<>1", "=a", "a*", "*c", "?b?", 1, "abc", "", "<>",
    "1", "2.5", "=2", "1E1", "x~*", "~?", "TRUE", "=FALSE",
]  # fmt: skip
# Criteria that are formulas rather than literals: a comparison operator joined to a date serial, i.e. a date as text.
CRITERIA_EXPR_POOL: list[str] = ['">"&DATE(2020,1,1)', '"<="&DATE(1900,1,1)']
# LET variables and LAMBDA parameters: none is a cell reference (`x1` is one), a function name (`N`) or `r`/`c`.
NAME_POOL: list[str] = ["x", "y", "z", "w", "v", "acc"]
# Functions passed by name as a LAMBDA (eta-reduced, `BYROW(A1:B3,SUM)`); Excel stores them `_xleta.SUM`
# and a bare `SUM` read from a file is #NAME? (docs/excel-semantics/xlsx-storage.md). They take 1 to 255
# arguments, so any arity fits (probes/m7-argument-modes: `MAKEARRAY(2,2,SUM)` is SUM(row,col)).
ETA_POOL: list[str] = ["SUM", "MAX", "MIN", "AVERAGE", "COUNT", "PRODUCT"]
# Share of a `pool` parameter's values drawn from its edge pool (synthetic-generator.md §3.2).
POOL_RATE = 0.35
MASK_OPS: list[str] = [">", "<", "=", "<>"]
# `--spill` (synthetic-generator.md §5.2 shape 3): the largest footprint reserved under a spilling probe, and the
# share of spilling probes given a blocker cell in it (edge class `spill_blocked`), so `#SPILL!` stays covered.
SPILL_ROWS = 12
SPILL_COLS = 6
SPILL_BLOCKED_RATE = 0.05
SPILL_BLOCKER = "x"
# The outer call of a generated formula: `=NAME(`, after any `_xlfn.`/`_xlws.` prefixes (`spelled`).
OUTER_CALL = re.compile(r"=(?:_xl[a-z]+\.)*([A-Z][A-Z0-9.]*)\(")


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

    def dynamic_array_names(self) -> set[str]:
        """Functions marked `dynamic_array: true`: their result can be a multi-cell array by design."""
        return {f["name"] for f in self.functions if f.get("dynamic_array")}

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


def pool_rate(p: dict[str, Any]) -> float:
    """Share of `p`'s values drawn from its pool: `pool_rate` when the catalog sets one (the D* database and
    criteria, whose Db pools only work together, T-2249), else POOL_RATE."""
    return float(p.get("pool_rate", POOL_RATE))


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
    # The case's pivot (sheet `Pivot`), built from its own seeded stream at the first `pivot: true` call (T-2396).
    pivot_seed: int = 0
    pivot: PivotSource | None = None

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
        return quoted(self.rng.choice(TEXT_POOL))

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
        return self.call_fn(self.weighted(fns or self.catalog.functions), depth)

    def call_fn(self, fn: dict[str, Any], depth: int) -> str:
        """A call to `fn` with generated arguments."""
        self.used.add(fn["name"])
        if fn.get("pivot"):
            # GETPIVOTDATA queries the case's pivot: its arguments are names from it, not free expressions.
            if self.pivot is None:
                self.pivot = build_source(random.Random(f"pivot-{self.pivot_seed}"))
            return pivot_call(self.rng, self.pivot, fn.get("spelled", fn["name"]))
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

    def pooled_lambda(self, p: dict[str, Any], arity: int) -> str | None:
        """A `lambda_body` pool entry of `arity` parameters (POOL_RATE of the time), with x and y renamed to
        free names so an outer LAMBDA or LET name is never shadowed; None to build one."""
        fitting = [t for t in POOLS.get(p.get("pool", ""), []) if lambda_arity(t) == arity]
        if not fitting or self.rng.random() >= POOL_RATE:
            return None
        text = self.rng.choice(fitting)
        names = {"x": self.fresh_name()}
        self.scope.append(names["x"])
        names["y"] = self.fresh_name()
        self.scope.pop()
        return re.sub(r"\b[xy]\b", lambda m: names[m.group(0)], text)

    def argument(self, p: dict[str, Any], depth: int, arity: int = 1) -> str:
        mode = p.get("mode", "value")
        if mode == "name":
            name = self.fresh_name()
            self.pending.append(name)
            return name
        if mode == "lambda":
            n = p.get("arity", 1)
            want = arity if n == "repeat" else int(n)
            return self.pooled_lambda(p, want) or self.lambda_arg(want, depth)
        if mode == "array" and p.get("mask"):
            # FILTER's include: one TRUE/FALSE per data row, so it fits a `full` array.
            text = f"({self.range_ref(full=True)}{self.rng.choice(MASK_OPS)}{self.literal('number')})"
        elif mode in ("array", "aggregate", "aggregate_a") and self.scope and self.rng.random() < 0.5:
            # A LAMBDA parameter or LET variable often holds the row, column or array (BYROW's SUM(x)).
            text = self.rng.choice(self.scope)
        elif mode == "range" and p.get("pool") and self.rng.random() < pool_rate(p):
            # A pooled `range` param takes array constants too (LOOKUP's and XMATCH's lookup_table).
            text = self.rng.choice(POOLS[p["pool"]])
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
            return self.criteria_text()
        if mode in ("aggregate", "aggregate_a") and self.rng.random() < 0.5:
            return self.range_ref()
        return self.scalar_text(p, depth)

    def criteria_text(self) -> str:
        """A `criteria` argument: a quoted text or numeric literal, or a date expression."""
        c = self.rng.choice(CRITERIA_POOL + CRITERIA_EXPR_POOL)
        if c in CRITERIA_EXPR_POOL:
            return str(c)
        return repr(c) if isinstance(c, int) else '"' + str(c) + '"'

    def scalar_text(self, p: dict[str, Any], depth: int) -> str:
        """A `value` param: an in-domain literal, else an edge-pool entry, else a free expression."""
        dom = p.get("domain") or {}
        in_domain = self.domain_value(dom)
        if in_domain is not None:
            return in_domain
        # A `hard` domain is never overridden: outside it Excel refuses the workbook (MUNIT).
        if p.get("pool") and not dom.get("hard") and self.rng.random() < pool_rate(p):
            return self.rng.choice(POOLS[p["pool"]])
        return self.value_expr(p, depth)

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


def narrow_targets(
    catalog: Catalog,
    targets: list[dict[str, Any]],
    milestone: str,
    functions: Sequence[str],
    category: str | None,
) -> list[dict[str, Any]]:
    """The `--focus` targets of `milestone` narrowed to the named `functions` and/or catalog `category`."""
    if functions:
        by_name = {str(f["name"]).upper(): f for f in catalog.functions}
        wanted = [n.strip().upper() for n in functions if n.strip()]
        unknown = [n for n in wanted if n not in by_name]
        if unknown:
            msg = f"unknown function(s) not in the catalog: {', '.join(unknown)}"
            raise ValueError(msg)
        elsewhere = [n for n in wanted if by_name[n].get("milestone") != milestone]
        if elsewhere:
            msg = f"function(s) outside milestone {milestone}: {', '.join(elsewhere)}"
            raise ValueError(msg)
        keep = set(wanted)
        missing = sorted(keep - {str(f["name"]).upper() for f in targets})
        if missing:
            msg = f"function(s) not generator-ready: {', '.join(missing)}"
            raise ValueError(msg)
        targets = [f for f in targets if str(f["name"]).upper() in keep]
    if category is not None:
        if not any(f.get("category") == category for f in catalog.functions):
            msg = f"unknown category: {category}"
            raise ValueError(msg)
        targets = [f for f in targets if f.get("category") == category]
    return targets


def generate_case(
    catalog: Catalog,
    seed: int,
    *,
    milestone: str,
    formulas: int = 40,
    data_rows: int = 12,
    data_cols: int = 5,
    max_depth: int = 3,
    focus: bool = False,
    case_index: int = 0,
    spill: bool = False,
    functions: Sequence[str] = (),
    category: str | None = None,
) -> dict[str, Any]:
    """One case. With `focus`, every formula's outer call is a function whose own milestone is `milestone`,
    taking turns through a seeded shuffle (each case reshuffles from its own seed; `case_index` offsets the
    position, so counts are exactly even within a case and roughly even across a run). With `spill`, a formula
    whose outer call is `dynamic_array` gets an empty footprint to spill into (`spill_layout`); the formulas
    themselves are the same as without it. `functions` (catalog names) and `category` narrow the `focus`
    targets; an unknown name, an empty selection or a function outside `milestone` is a ValueError."""
    rng = random.Random(seed)
    g = Gen(rng, catalog.scoped(milestone), data_rows, data_cols, max_depth, pivot_seed=seed)
    targets = [f for f in g.catalog.functions if f.get("milestone") == milestone] if focus else []
    if (functions or category is not None) and not focus:
        msg = "--functions and --category narrow --focus; pass --focus too"
        raise ValueError(msg)
    if focus:
        targets = narrow_targets(catalog, targets, milestone, functions, category)
        if not targets:
            msg = f"no generator-ready functions at milestone {milestone} to focus on"
            raise ValueError(msg)
        rng.shuffle(targets)
    data: dict[str, Any] = {}
    for r in range(1, data_rows + 1):
        for c in range(data_cols):
            v = g.data_value()
            if v is not None:
                if isinstance(v, str) and v.startswith("="):
                    v = {"t": "s", "v": v}
                data[f"{chr(ord('A') + c)}{r}"] = v
    texts: list[str] = []
    for i in range(1, formulas + 1):
        if focus:
            texts.append("=" + g.call_fn(targets[(case_index * formulas + i - 1) % len(targets)], 0))
            continue
        typ = rng.choice(["number", "number", "any", "text", "bool"])
        texts.append("=" + g.expr(typ, 0))
    tags = ["generated", milestone, *sorted(g.used)]
    if spill:
        # Its own stream: the layout never shifts the formulas' draws.
        layout = spill_layout(
            texts,
            catalog.dynamic_array_names(),
            rows=min(SPILL_ROWS, data_rows),
            cols=min(SPILL_COLS, data_cols),
            rng=random.Random(f"spill-{seed}"),
        )
        calc, check = layout.cells, layout.check
        tags += ["spill", *(["spill_blocked"] if layout.blocked else [])]
    else:
        calc = {f"A{i}": text for i, text in enumerate(texts, 1)}
        check = [f"Calc!A{i}" for i in range(1, formulas + 1)]
    return {
        "format": "rustsheet-case/v1",
        "description": f"generated P0 seed={seed} milestone={milestone}" + (" spill" if spill else ""),
        "tags": tags,
        "sheets": [
            {"name": "Data", "cells": data},
            {"name": "Calc", "cells": calc},
            *([g.pivot.spec_sheet()] if g.pivot is not None else []),
            *([database.spec_sheet()] if database.uses_db(texts) else []),
        ],
        "expected": {},
        "check": check,
    }


@dataclass
class SpillLayout:
    """The Calc sheet of a `--spill` case: its cells (formulas and blockers), the cells to check, and how
    many footprints have a blocker."""

    cells: dict[str, Any]
    check: list[str]
    blocked: int


def spill_layout(
    texts: list[str], dynamic: set[str], *, rows: int, cols: int, rng: random.Random
) -> SpillLayout:
    """Formulas down column A, each spilling one (outer call in `dynamic`) followed by the empty `rows` x `cols`
    footprint it anchors (A..). Nothing else is in a footprint's rows, so footprints never overlap a formula or
    each other. Every footprint cell is checked, so a cell Excel leaves empty is scored as empty. A result
    taller than the footprint is `#SPILL!` against the next formula below. A wider one spills right into
    unchecked cells, where the harness's spill-extent check fails an engine value Excel left empty.
    `SPILL_BLOCKED_RATE` of the footprints get a blocker constant just below or right of the anchor
    (`spill_blocked`): an input, not checked."""
    cells: dict[str, Any] = {}
    check: list[str] = []
    blocked = 0
    row = 1
    for text in texts:
        cells[f"A{row}"] = text
        m = OUTER_CALL.match(text)
        if not (m and m.group(1) in dynamic):
            check.append(f"Calc!A{row}")
            row += 1
            continue
        footprint = [f"{col_letter(c)}{r}" for r in range(row, row + rows) for c in range(cols)]
        blocker = None
        if rng.random() < SPILL_BLOCKED_RATE and len(footprint) > 1:
            below, right = f"A{row + 1}", f"B{row}"
            blocker = rng.choice([a for a in (below, right) if a in footprint])
            cells[blocker] = SPILL_BLOCKER
            blocked += 1
        check += [f"Calc!{a1}" for a1 in footprint if a1 != blocker]
        row += rows
    return SpillLayout(cells, check, blocked)


def write_case(spec: dict[str, Any], out_dir: Path) -> Path:
    body = json.dumps(spec, indent=1, sort_keys=True) + "\n"
    digest = hashlib.sha256(body.encode()).hexdigest()[:16]
    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / f"{digest}.case.json"
    path.write_text(body)
    return path
