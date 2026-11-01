"""Policy `numeric-rules`: dense probe batches for Excel's floating-point rules.

Each probe is one formula cell on sheet `Calc`; operands that must be exact doubles live as
formulas on sheet `Inputs` (built from powers of two, which Excel computes exactly) so the
probe is not confounded by Excel's 15-digit literal parsing. Every probe carries a hypothesis
tag in the case's `hypotheses` map (cell -> `family/form/params`). Expectations stay empty:
run `rustsheet-oracle fill` (T-0004). Design: docs/architecture/synthetic-generator.md
§5.2/§6.1; hypotheses: docs/excel-semantics/numbers.md.
"""

from __future__ import annotations

import random
from typing import Any

# Composite bases are parenthesized: unary minus binds tighter than ^ in Excel (-2^20 = +2^20).
# Powers of two give exact `base*(1 ± k*2^-52)` (k is a true ULP count); 3, 100.5 and 1.5 round
# in the Inputs cell, so for them k is only nominal. Either way the oracle measures Excel.
BASES = ["1", "2", "4", "8", "0.5", "0.25", "1024", "(2^20)", "(2^-10)", "3", "100.5", "1.5"]
SNAP_K = range(9)
SNAP_FORMS = {
    "top-level-sub": "={x}-{b}",
    "parenthesized-sub": "=({x}-{b})",
    "sum-fn": "=SUM({x},-{b})",
    "add-negated": "={x}+(-{b})",
    "sub-then-add-zero": "={x}-{b}+0",
    "sub-then-mul-one": "=({x}-{b})*1",
}
CMP_OPS = ["=", "<", ">", "<=", ">=", "<>"]
# FP-trap expressions for the numbers.md hypotheses 'Number -> text uses 15 significant digits'
# and 'Near-zero snapping' (Microsoft's documented 43.1-43.2 and 1.333+1.225 examples, 15-digit
# truncation, 2^53 boundaries, overflow/denormal producers).
TRAPS = [
    "0.1+0.2",
    "0.1+0.2-0.3",
    "43.1-43.2",
    "(43.1-43.2)+1",
    "1/3",
    "2/3",
    "1.1*1.1",
    "0.1*3",
    "1.333+1.225-1.333-1.225",
    "1e-15",
    "1+1e-15",
    "1+2^-52",
    "0.000123456789012345",
    "1e21",
    "1e15+0.3",
    "999999999999999",
    "9999999999999999",
    "2^53+1",
    "2^-1074",
    "1.005*1000",
    "0.285*100",
    "4.35*100",
    "1-0.9",
    "0.3-0.1",
    "1e-5",
    "123456789012345678",
    "1/7",
    "2^0.5",
    "10/3",
    "-1/3",
    "1e307*100",  # 1e308+ literals make Excel reject the workbook (max typeable 9.99999999999999E+307)
    "1e-307/10",
    "0.1^2",
    "100*1.1",
    "0.7+0.1",
    "8.2-0.2",
]
TEXT_FORMS = {
    "concat": "=CONCAT({e})",
    "amp-empty": '=({e})&""',
    "text-18dp": '=TEXT({e},"0.000000000000000000")',
    "len-of-text": '=LEN(({e})&"")',
}


def fmt_k(sign: int, k: int) -> str:
    """The ULP offset `±k*2^-52` inside `base*(1...)`, empty for k = 0."""
    return "" if k == 0 else f"{'+' if sign > 0 else '-'}{k}*2^-52"


def _random_literal(rng: random.Random) -> str:
    digits = rng.randint(14, 20)
    body = "".join(rng.choice("0123456789") for _ in range(digits))
    body = str(rng.randint(1, 9)) + body[1:]
    pos = rng.randint(1, digits)
    mantissa = f"{body[:pos]}.{body[pos:]}" if pos < digits else body
    exp = rng.choice(["", "", f"E{rng.randint(-10, 10)}"])
    return mantissa + exp


def generate_numeric_rules(seed: int) -> list[dict[str, Any]]:
    """Return `rustsheet-case/v1` specs (one per family) with >= 2,000 probe cells total."""
    rng = random.Random(seed)
    cases: list[dict[str, Any]] = []

    def add(family: str, inputs: dict[str, str], probes: list[tuple[str, str]]) -> None:
        calc = {f"A{i}": f for i, (f, _) in enumerate(probes, 1)}
        sheets = [{"name": "Calc", "cells": calc}]
        if inputs:
            sheets.insert(0, {"name": "Inputs", "cells": inputs})
        cases.append(
            {
                "format": "rustsheet-case/v1",
                "description": f"numeric-rules probes ({family}) seed={seed}; hypotheses in numbers.md",
                "tags": ["probe", "numeric-rules", family],
                "sheets": sheets,
                "expected": {},
                "check": [f"Calc!{c}" for c in calc],
                "hypotheses": {f"Calc!A{i}": f"{family}/{t}" for i, (_, t) in enumerate(probes, 1)},
            }
        )

    # near-zero snapping in +/- (hypothesis: numbers.md "Near-zero snapping")
    inputs: dict[str, str] = {}
    probes: list[tuple[str, str]] = []
    for base in BASES:
        for sign in (1, -1):
            for k in SNAP_K:
                cell = f"A{len(inputs) + 1}"
                inputs[cell] = f"={base}*(1{fmt_k(sign, k)})"
                for form, tmpl in SNAP_FORMS.items():
                    f = tmpl.format(x=f"Inputs!{cell}", b=base)
                    probes.append((f, f"{form}/base={base}/sign={sign:+d}/k={k}"))
    add("snap-addsub", inputs, probes)

    # comparison tolerance
    inputs, probes = {}, []
    for base in BASES:
        for k in SNAP_K:
            cell = f"A{len(inputs) + 1}"
            inputs[cell] = f"={base}*(1+{k}*2^-52)"
            for op in CMP_OPS:
                probes.append((f"=Inputs!{cell}{op}{base}", f"op={op}/base={base}/k={k}"))
    add("compare-tolerance", inputs, probes)

    # 15-digit literal parsing, as formula literal, text coercion and VALUE()
    probes = []
    for _ in range(150):
        lit = _random_literal(rng)
        probes.append((f"={lit}", f"literal/{lit}"))
        probes.append((f'="{lit}"+0', f"text-coerce/{lit}"))
        probes.append((f'=VALUE("{lit}")', f"value-fn/{lit}"))
    add("literal-15-digit", {}, probes)

    # number -> text conversion
    probes = [(tmpl.format(e=e), f"{form}/{e}") for e in TRAPS for form, tmpl in TEXT_FORMS.items()]
    for _ in range(100):
        a, b = rng.randint(1, 999), rng.randint(2, 999)
        probes.append((f"=CONCAT({a}/{b})", f"concat/{a}/{b}"))
        probes.append((f'=({a}/{b}*{b})&""', f"amp-empty-roundtrip/{a}/{b}"))
    add("number-to-text", {}, probes)

    return cases
