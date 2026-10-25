"""Verifier (T-2586, verify-r5-f-bigconst): own parser for F.DIST(x,d1,d2,FALSE) probe cases.
Reads <case>.case.json + <case>.oracle.json from probes-numeric, evaluates the x formulas exactly (Fractions; decimal
literals truncated to 15 significant digits, as Excel does), checks Excel's echo of x, and returns rows
{cell, x, d1, d2, excel, echo_ok}."""

from __future__ import annotations

import ast
import json
import re
from fractions import Fraction
from pathlib import Path

CASES = Path(__file__).resolve().parents[4] / "eval" / "suites" / "probes-numeric" / "cases"
FD = re.compile(r"^=F\.DIST\(A(\d+),([0-9.]+),([0-9.]+),FALSE\)$")


def lit(s: str) -> Fraction:
    """Excel number literal, truncated (not rounded) to 15 significant digits."""
    digits = s.replace(".", "").lstrip("0")
    if len(digits) <= 15:
        return Fraction(s)
    intpart, _, frac = s.partition(".")
    allds = intpart + frac
    lead = len(allds) - len(allds.lstrip("0"))
    keep = allds[: lead + 15]
    scale = len(allds) - len(intpart)
    return Fraction(int(keep) * 10 ** (len(allds) - len(keep)), 10**scale)


def ev(node) -> Fraction:
    if isinstance(node, ast.Expression):
        return ev(node.body)
    if isinstance(node, ast.Constant):
        return lit(repr(node.value)) if isinstance(node.value, float) else Fraction(node.value)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        return -ev(node.operand)
    if isinstance(node, ast.BinOp):
        lv, rv = ev(node.left), ev(node.right)
        if isinstance(node.op, ast.Add):
            return lv + rv
        if isinstance(node.op, ast.Sub):
            return lv - rv
        if isinstance(node.op, ast.Mult):
            return lv * rv
        if isinstance(node.op, ast.Pow):  # Excel ^ (integer exponents only here)
            assert rv.denominator == 1
            return lv ** int(rv)
    raise ValueError(ast.dump(node))


def xval(formula: str) -> float:
    """Exact value of an A-column formula, rounded once to double. Every probe x is an exact double by design."""
    # Excel ^ binds tighter than *; the probes only use literal a^-k, so it maps to Python **.
    q = ev(ast.parse(formula.lstrip("=").replace("^", "**"), mode="eval"))
    v = float(q)
    assert Fraction(v) == q, formula
    return v


def rows(case: str) -> list[dict]:
    cells = json.loads((CASES / f"{case}.case.json").read_text())["sheets"][0]["cells"]
    orc = json.loads((CASES / f"{case}.oracle.json").read_text())["sheets"]["Calc"]
    out = []
    for ref, f in cells.items():
        m = FD.match(f)
        if not m:
            continue
        a = f"A{m.group(1)}"
        x = xval(cells[a])
        o, e = orc.get(ref), orc.get(a)
        if not o or o.get("t") != "n":
            continue
        out.append(dict(cell=ref, x=x, d1=float(m.group(2)), d2=float(m.group(3)), excel=float(o["v"]),
                        echo_ok=(e is not None and float(e["v"]) == x)))
    return out
