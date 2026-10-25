"""Independent loader for the T-2513 r3-loader-bd0 probes (verifier).

Parses each case formula `=BETA.DIST(x, a, b, FALSE)` with a small expression evaluator (numbers, + - * / ^,
parentheses, unary minus) in IEEE double, truncating every numeric literal to 15 significant digits as Excel does,
reads Excel's value from the oracle sidecar, and groups cells into blocks = maximal runs of consecutive cells with
the same (a, b) whose x values are consecutive doubles.  Nothing here comes from the worker's meta files.
"""
from __future__ import annotations

import json
import math
import re
from decimal import Decimal, ROUND_DOWN
from pathlib import Path

CASES = Path("/home/user/rustsheet/eval/suites/probes-numeric/cases")
TOK = re.compile(r"\s*(\d+\.?\d*(?:[eE][+-]?\d+)?|\.\d+(?:[eE][+-]?\d+)?|[-+*/^(),]|[A-Za-z_.]+)")


def lit15(s: str) -> float:
    """Excel keeps at most 15 significant digits of a typed literal (truncation)."""
    d = Decimal(s)
    if d == 0:
        return 0.0
    exp = d.adjusted() - 14
    return float(d.quantize(Decimal(1).scaleb(exp), rounding=ROUND_DOWN)) if len(d.as_tuple().digits) > 15 else float(d)


class P:
    def __init__(self, s: str):
        self.t = TOK.findall(s)
        self.i = 0

    def peek(self):
        return self.t[self.i] if self.i < len(self.t) else None

    def eat(self, x=None):
        v = self.t[self.i]
        if x is not None and v != x:
            raise ValueError(f"expected {x}, got {v}")
        self.i += 1
        return v

    def expr(self):
        v = self.term()
        while self.peek() in ("+", "-"):
            op = self.eat()
            w = self.term()
            v = v + w if op == "+" else v - w
        return v

    def term(self):
        v = self.power()
        while self.peek() in ("*", "/"):
            op = self.eat()
            w = self.power()
            v = v * w if op == "*" else v / w
        return v

    def power(self):  # Excel: unary minus binds tighter than ^
        v = self.unary()
        while self.peek() == "^":
            self.eat()
            v = float(v) ** self.unary()
        return v

    def unary(self):
        if self.peek() == "-":
            self.eat()
            return -self.unary()
        if self.peek() == "+":
            self.eat()
            return self.unary()
        if self.peek() == "(":
            self.eat("(")
            v = self.expr()
            self.eat(")")
            return v
        return lit15(self.eat())


def parse_call(f: str):
    """'=BETA.DIST(x,a,b,FALSE)' -> (x, a, b, cum)."""
    p = P(f.lstrip("="))
    name = p.eat()
    if name != "BETA.DIST":
        raise ValueError(name)
    p.eat("(")
    args = [p.expr()]
    for _ in range(2):
        p.eat(",")
        args.append(p.expr())
    p.eat(",")
    cum = p.eat()
    p.eat(")")
    if p.peek() is not None:
        raise ValueError(f)
    return float(args[0]), float(args[1]), float(args[2]), cum.upper() == "TRUE"


def load_case(cid: str):
    case = json.loads((CASES / f"{cid}.case.json").read_text())
    side = CASES / f"{cid}.oracle.json"
    vals = json.loads(side.read_text())["sheets"]["Calc"] if side.exists() else {}
    cells = case["sheets"][0]["cells"]
    keys = sorted(cells, key=lambda c: int(c[1:]))
    rows = []
    for c in keys:
        x, a, b, cum = parse_call(cells[c])
        v = vals.get(c)
        rows.append(dict(case=cid, cell=c, x=x, a=a, b=b, cum=cum,
                         excel=float(v["v"]) if v and v.get("t") == "n" else None))
    blk, prev = 0, None
    for r in rows:
        if not (prev and prev["a"] == r["a"] and prev["b"] == r["b"] and math.nextafter(prev["x"], 2) == r["x"]):
            blk += 1
        r["blk"] = f"{cid.rsplit('-', 1)[-1]}#{blk}"
        prev = r
    return rows


def blocks(rows):
    out: dict = {}
    for i, r in enumerate(rows):
        out.setdefault(r["blk"], []).append(i)
    return out
