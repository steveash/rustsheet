"""`rustsheet-gen --policy arg-pairs` (T-3278): calls that pass the argument-class pairs no Excel cell covers yet.

The eval report lists, per function, the pairs of two catalog parameters and an argument class of each that no
Excel-evidence call has passed together (`argument_coverage[].unseen_pairs`, T-3277). This policy writes one or
more calls per such pair, each forcing the two arguments to their classes and drawing every other argument as a
plain in-domain value, so the pair decides the result. Classes are read from the report, never recomputed here:
the classifier (crates/rustsheet-eval/src/argument_coverage.rs) is their one definition. A forced argument is a
literal or a reference to a Data cell holding the class's value, the two shapes the classifier reads.

Design: docs/architecture/synthetic-generator.md §6.7.
"""

from __future__ import annotations

import json
import random
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rustsheet_tools.gen.generator import Catalog, Gen, narrow_targets
from rustsheet_tools.gen.pools import POOLS

# Calls per pair: the first forces each class as a literal, the second as a reference to a Data cell holding
# another value of the class, so two different values per class reduce accidental agreement.
CALLS_PER_PAIR = 2
# Formulas per case, so one oracle workbook stays small.
MAX_FORMULAS = 200
# Rows between formulas on the Calc sheet: a forced `range` argument of a value parameter is at most two rows
# tall, so the array it lifts to spills into the gap instead of onto the next formula.
ROW_STRIDE = 3
DATA_ROWS = 12
DATA_COLS = 5
# Column G holds one Data cell per class (the second call's value); column H stays empty for `blank`.
CLASS_COLUMN = "G"
BLANK_COLUMN = "H"
# The pooled error literals (`#N/A`, ...), not the error-producing expressions (`1/0`), which the classifier
# does not read.
ERROR_LITERALS = [e for e in POOLS["errors"] if e.startswith("#")]

# Per class (the report's snake_case names, ALL_CLASSES order): the first call's literal and the second call's
# Data cell value (spec2xlsx cell input). `error`'s literal is drawn from ERROR_LITERALS, `blank` references an
# empty cell both times, and `range` is a multi-cell reference both times.
CLASS_FORMS: dict[str, tuple[str, Any]] = {
    "number": ("2", 7),
    "zero": ("0", 0),
    "negative": ("-1", -2.5),
    "fraction": ("0.5", 1.5),
    "numeric_text": ('"2.5"', {"t": "s", "v": "12"}),
    "text": ('"abc"', {"t": "s", "v": "x"}),
    "empty_text": ('""', ""),
    "boolean": ("TRUE", False),
    "error": ("#N/A", {"t": "e", "v": "#DIV/0!"}),
    "blank": (f"Data!{BLANK_COLUMN}1", None),
    "range": ("Data!A1:B2", None),
}
CLASSES = list(CLASS_FORMS)
# The second call's multi-cell reference for `range` (2 x 2, so a lifted result fits in the row gap).
RANGE_SECOND = "Data!C3:D4"


def class_cell(cls: str) -> str:
    """The Data cell holding `cls`'s second-call value."""
    return f"{CLASS_COLUMN}{CLASSES.index(cls) + 1}"


@dataclass(frozen=True)
class Pair:
    """One unseen pair from the report: two catalog parameter names, in parameter order, and a class of each."""

    params: tuple[str, str]
    classes: tuple[str, str]

    def label(self) -> str:
        return f"{self.params[0]}={self.classes[0]}/{self.params[1]}={self.classes[1]}"


def load_unseen_pairs(report: Path) -> dict[str, list[Pair]]:
    """`argument_coverage[].unseen_pairs` of a report.json, by upper-case function name."""
    data = json.loads(report.read_text(encoding="utf-8"))
    out: dict[str, list[Pair]] = {}
    for row in data.get("argument_coverage", []):
        pairs = [
            Pair((str(u["params"][0]), str(u["params"][1])), (str(u["classes"][0]), str(u["classes"][1])))
            for u in row.get("unseen_pairs", [])
        ]
        if pairs:
            out[str(row["name"]).upper()] = pairs
    return out


def plain_value(p: dict[str, Any], rng: random.Random) -> str:
    """An unforced `value` argument: in its domain when it has one, else a plain literal of its type."""
    dom = p.get("domain") or {}
    if dom.get("int"):
        return str(rng.randint(dom.get("min", 0), dom.get("max", 10)))
    if dom.get("enum"):
        v = rng.choice(dom["enum"])
        return str(v) if isinstance(v, int | float) and not isinstance(v, bool) else '"' + str(v) + '"'
    typ = p.get("type", "any")
    if typ == "text":
        return rng.choice(['"abc"', '"Hello"'])
    if typ == "bool":
        return rng.choice(["TRUE", "FALSE"])
    return rng.choice(["1", "2", "3"])


def skip_reason(fn: dict[str, Any], pair: Pair) -> str | None:
    """Why `pair` cannot be forced on `fn`, or None. A `hard` domain is never left: outside it Excel refuses the
    workbook (MUNIT), so a pair forcing such a parameter is skipped."""
    names = [p["name"] for p in fn["params"]]
    for name in pair.params:
        if name not in names:
            return f"no parameter {name}"
        if (fn["params"][names.index(name)].get("domain") or {}).get("hard"):
            return "hard domain"
    if any(c not in CLASS_FORMS for c in pair.classes):
        return "unknown class"
    return None


class ArgPairs:
    """The calls of one function's pairs, drawn from a seeded `Gen` (for its range, name and lambda arguments)."""

    def __init__(self, gen: Gen) -> None:
        self.gen = gen

    def forced(self, p: dict[str, Any], cls: str, variant: int) -> str:
        """Parameter `p`'s argument of class `cls` for call `variant` (0 literal, 1 Data reference)."""
        if cls == "range":
            if p.get("mode") == "range":
                # A range parameter counts any reference as `range`: its own shaped draw (vector, same_shape).
                return self.gen.argument(p, 0)
            return CLASS_FORMS["range"][0] if variant == 0 else RANGE_SECOND
        if cls == "blank":
            return f"Data!{BLANK_COLUMN}{variant + 1}"
        if variant == 1:
            return f"Data!{class_cell(cls)}"
        if cls == "error":
            return self.gen.rng.choice(ERROR_LITERALS)
        return str(CLASS_FORMS[cls][0])

    def unforced(self, p: dict[str, Any]) -> str:
        if p.get("mode", "value") in ("value", "criteria"):
            return plain_value(p, self.gen.rng)
        # Ranges, arrays, aggregates, names and lambdas keep the generator's own draw (shapes, scope).
        return self.gen.argument(p, 0)

    def call(self, fn: dict[str, Any], pair: Pair, variant: int) -> str:
        """`fn` called with `pair` forced: each parameter once (one repeat group), through the later forced one
        and every required or `always` parameter, so the classifier's `param_of` maps argument i to param i."""
        params: list[dict[str, Any]] = fn["params"]
        names = [p["name"] for p in params]
        force = {pair.params[0]: pair.classes[0], pair.params[1]: pair.classes[1]}
        last = max(
            [names.index(n) for n in force]
            + [i for i, p in enumerate(params) if not p.get("optional") or p.get("always")]
        )
        gen = self.gen
        outer_scope, outer_pending = len(gen.scope), gen.pending
        gen.pending, gen.call_shape = [], None
        args = []
        for p in params[: last + 1]:
            if p.get("body"):
                gen.scope += gen.pending
                gen.pending = []
            if p["name"] in force:
                args.append(self.forced(p, force[p["name"]], variant))
                gen.scope += gen.pending
                gen.pending = []
            else:
                args.append(self.unforced(p))
        del gen.scope[outer_scope:]
        gen.pending = outer_pending
        return f"{fn.get('spelled', fn['name'])}({','.join(args)})"


def data_sheet(gen: Gen) -> dict[str, Any]:
    """The Data sheet: a DATA_ROWS x DATA_COLS block of generator values (the ranges' contents), one cell per
    class in CLASS_COLUMN and an empty BLANK_COLUMN."""
    cells: dict[str, Any] = {}
    for r in range(1, DATA_ROWS + 1):
        for c in range(DATA_COLS):
            v = gen.data_value()
            if v is not None:
                cells[f"{chr(ord('A') + c)}{r}"] = (
                    {"t": "s", "v": v} if isinstance(v, str) and v.startswith("=") else v
                )
    for cls, (_, value) in CLASS_FORMS.items():
        if value is not None:
            cells[class_cell(cls)] = value
    return cells


def generate_arg_pairs(
    catalog: Catalog,
    unseen: dict[str, list[Pair]],
    seed: int,
    *,
    milestone: str,
    functions: Sequence[str] = (),
    calls_per_pair: int = CALLS_PER_PAIR,
) -> tuple[list[dict[str, Any]], dict[str, list[str]]]:
    """Cases whose calls pass the `unseen` pairs of the generator-ready functions of exactly `milestone` (or the
    named `functions`), CALLS_PER_PAIR calls per pair, MAX_FORMULAS formulas per case. Each function draws
    from its own stream (`seed` and its name), so the same seed and report give the same cases. A pair that
    cannot be forced (`skip_reason`) is named in its case's description and returned by function (the second
    value), so a function whose every pair is skipped, and so has no case, is still reported."""
    scoped = catalog.scoped(milestone)
    own = [f for f in scoped.functions if f.get("milestone") == milestone]
    targets = [
        f
        for f in narrow_targets(catalog, own, milestone, functions, None)
        # GETPIVOTDATA's arguments are names from a pivot, and a LAMBDA must be invoked: neither is a plain call.
        if not f.get("pivot") and not f.get("invoke")
    ]
    specs: list[dict[str, Any]] = []
    skipped_by_fn: dict[str, list[str]] = {}
    for fn in sorted(targets, key=lambda f: str(f["name"])):
        name = str(fn["name"]).upper()
        pairs = unseen.get(name, [])
        if not pairs:
            continue
        gen = Gen(random.Random(f"arg-pairs-{seed}-{name}"), scoped, DATA_ROWS, DATA_COLS, 1)
        data = data_sheet(gen)
        maker = ArgPairs(gen)
        skipped = [f"{pair.label()} ({why})" for pair in pairs if (why := skip_reason(fn, pair)) is not None]
        if skipped:
            skipped_by_fn[name] = skipped
        texts = [
            "=" + maker.call(fn, pair, variant)
            for pair in pairs
            if skip_reason(fn, pair) is None
            for variant in range(calls_per_pair)
        ]
        chunks = [texts[i : i + MAX_FORMULAS] for i in range(0, len(texts), MAX_FORMULAS)]
        for part, chunk in enumerate(chunks, 1):
            note = f"; skipped: {', '.join(skipped)}" if skipped else "; skipped: none"
            calc = {f"A{1 + i * ROW_STRIDE}": text for i, text in enumerate(chunk)}
            specs.append(
                {
                    "format": "rustsheet-case/v1",
                    "description": (
                        f"generated arg-pairs seed={seed} milestone={milestone} {name} part {part}/{len(chunks)}: "
                        f"{len(chunk)} calls over {len(pairs) - len(skipped)} unseen pairs{note}"
                    ),
                    "tags": ["generated", "arg-pairs", milestone, name],
                    "sheets": [{"name": "Data", "cells": data}, {"name": "Calc", "cells": calc}],
                    "expected": {},
                    "check": [f"Calc!{a1}" for a1 in calc],
                }
            )
    return specs, skipped_by_fn
