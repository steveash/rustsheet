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

    # -- references into the data sheet --------------------------------------------------
    def cell_ref(self) -> str:
        col = col_letter(self.rng.randrange(self.data_cols))
        row = self.rng.randrange(1, self.data_rows + 1)
        dollar_c = "$" if self.rng.random() < 0.2 else ""
        dollar_r = "$" if self.rng.random() < 0.2 else ""
        return f"Data!{dollar_c}{col}{dollar_r}{row}"

    def range_ref(self, width: int = 1) -> str:
        c0 = self.rng.randrange(self.data_cols - width + 1)
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
        i = 0
        while i < len(params):
            p = params[i]
            # Excel allows omitting trailing optionals only.
            if p.get("optional") and self.rng.random() < 0.4 and all(q.get("optional") for q in params[i:]):
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
            args.extend(self.argument(p, depth) for _ in range(count))
            i += 1
        return f"{fn['name']}({','.join(args)})"

    def argument(self, p: dict[str, Any], depth: int) -> str:
        mode = p.get("mode", "value")
        if mode == "range" or (mode == "array" and self.rng.random() < 0.5):
            return self.range_ref(width=self.rng.randint(1, min(4, self.data_cols)))
        if mode == "criteria":
            c = self.rng.choice(CRITERIA_POOL)
            return repr(c) if isinstance(c, int) else '"' + str(c) + '"'
        if mode in ("aggregate", "aggregate_a") and self.rng.random() < 0.5:
            return self.range_ref()
        dom = p.get("domain") or {}
        # A `hard` domain is never violated: outside it Excel refuses the workbook (MUNIT(10^6)).
        if dom.get("int") and (dom.get("hard") or self.rng.random() < 0.7):
            return str(self.rng.randint(dom.get("min", 0), dom.get("max", 10)))
        return self.expr(p.get("type", "any"), depth + 1)

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
