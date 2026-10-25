"""verify-r5-f-edges: independent parser for the F.DIST(x,d1,d2,FALSE) probe cases (no worker code).

Each row r: A_r = an exact x formula, B_r = F.DIST(A_r,d1,d2,FALSE), optional C_r = B*2^k, optional D_r = twin.
x is evaluated here from A_r's formula (Graph reports a subnormal x as 0); numeric literals are cut to 15
significant digits (truncated, as Excel does) before evaluation."""
from __future__ import annotations

import json
import math
import re
from decimal import Decimal
from fractions import Fraction
from pathlib import Path

CASES = Path(__file__).resolve().parents[4] / "eval/suites/probes-numeric/cases"
NUM = re.compile(r"\d+(?:\.\d*)?(?:E[+-]?\d+)?", re.I)


def lit(s: str) -> Fraction:
    """A formula literal as Excel reads it: truncated to 15 significant digits."""
    d = Decimal(s)
    if d == 0:
        return Fraction(0)
    sgn, digits, exp = d.as_tuple()
    keep = digits[:15]
    exp += len(digits) - len(keep)
    return Fraction(int("".join(map(str, keep)))) * (Fraction(10) ** exp)


def rnd(q: Fraction) -> float:
    """Correctly rounded double of an exact rational (subnormals kept, IEEE)."""
    if q == 0:
        return 0.0
    return q.numerator / q.denominator if abs(q.numerator) < 2**1000 and q.denominator < 2**1000 else _rnd_big(q)


def _rnd_big(q: Fraction) -> float:
    import mpmath as mp
    with mp.workprec(200):
        v = mp.mpf(q.numerator) / q.denominator
    return float(mp.mpf(v))  # 200 bits then to double: double rounding chance ~2^-147, ignorable


def flush(v: float) -> float:
    """Excel's `*`, `/` and `^` return a true 0 for a subnormal result (numbers.md, T-0989); `-`, `+` and EXP keep it."""
    return 0.0 if 0 < abs(v) < 2.0**-1022 else v


def ev(expr: str) -> float:
    """Evaluate the small exact expressions used in these probes: + - * ^ ( ) EXP(), left-assoc, ^ highest.
    Every intermediate is rounded to a double (Excel's per-op doubles), a subnormal product, quotient or
    power is flushed to 0 (as Excel does); EXP uses Python's exp (only on
    probe x cells at EXP(-7xx), which only need 'is subnormal or 0')."""
    toks = re.findall(r"EXP|\d+(?:\.\d*)?(?:E[+-]?\d+)?|[-+*/^()]", expr.replace(" ", ""), re.I)
    pos = 0

    def peek():
        return toks[pos] if pos < len(toks) else None

    def take():
        nonlocal pos
        pos += 1
        return toks[pos - 1]

    def atom():
        t = take()
        if t == "(":
            v = add()
            assert take() == ")"
            return v
        if t == "-":
            return -power()
        if t.upper() == "EXP":
            assert take() == "("
            v = add()
            assert take() == ")"
            return math.exp(v)
        return rnd(lit(t))

    def power():
        v = atom()
        while peek() == "^":
            take()
            e = atom()
            v = flush(rnd(Fraction(v) ** int(e)) if float(e).is_integer() else v ** e)
        return v

    def mul():
        v = power()
        while peek() in ("*", "/"):
            op = take()
            w = power()
            v = flush(rnd(Fraction(v) * Fraction(w)) if op == "*" else rnd(Fraction(v) / Fraction(w)))
        return v

    def add():
        v = mul()
        while peek() in ("+", "-"):
            op = take()
            w = mul()
            v = rnd(Fraction(v) + Fraction(w)) if op == "+" else rnd(Fraction(v) - Fraction(w))
        return v

    out = add()
    assert pos == len(toks), expr
    return out


CALL = re.compile(r"^=F\.DIST\(A(\d+),([^,]+),([^,]+),FALSE\)(?:\*2\^(\d+))?$")


def oval(o):
    if o is None:
        return None
    return o["v"] if o["t"] == "e" else float(o["v"])


def rows(case: str) -> list[dict]:
    c = json.loads((CASES / f"{case}.case.json").read_text())["sheets"][0]["cells"]
    op = CASES / f"{case}.oracle.json"
    o = json.loads(op.read_text())["sheets"]["Calc"] if op.exists() else {}
    nrow = max(int(k[1:]) for k in c if k[0] == "A")
    out = []
    for r in range(1, nrow + 1):
        fa = c.get(f"A{r}")
        fb = c.get(f"B{r}")
        if fa is None or fb is None:
            continue
        mb = CALL.match(fb)
        assert mb and int(mb.group(1)) == r, fb
        x = ev(fa[1:])
        d1, d2 = ev(mb.group(2)), ev(mb.group(3))
        b = oval(o.get(f"B{r}"))
        cc = c.get(f"C{r}")
        k = None
        cval = None
        if cc:
            mc = CALL.match(cc)
            assert mc and mc.group(2) == mb.group(2) and mc.group(3) == mb.group(3)
            k = int(mc.group(4))
            cval = oval(o.get(f"C{r}"))
        excel = b
        if isinstance(cval, float) and math.isfinite(cval):
            excel = rnd(Fraction(cval) / 2**k) if cval != 0 else (0.0 if b == 0 else b)
            # a nonzero C with B = 0: Graph's subnormal report; keep the exact C/2^k
        twin = None
        dd = c.get(f"D{r}")
        if dd:
            md = CALL.match(dd)
            twin = dict(d1=ev(md.group(2)), d2=ev(md.group(3)), excel=oval(o.get(f"D{r}")))
        aecho = oval(o.get(f"A{r}"))
        out.append(dict(row=r, fa=fa, x=x, d1=d1, d2=d2, excel=excel, B=b, C=cval, k=k, twin=twin, aecho=aecho))
    return out


if __name__ == "__main__":
    import sys
    for case in sys.argv[1:]:
        rs = rows(case)
        bad = [r for r in rs if isinstance(r["aecho"], float) and r["aecho"] != 0 and r["aecho"] != r["x"]]
        sub0 = [r for r in rs if r["aecho"] == 0 and r["x"] != 0]
        cb = [r for r in rs if r["C"] not in (None, 0) and isinstance(r["C"], float) and r["B"] == 0]
        print(case, len(rs), "rows; echo mismatch", len(bad), "; x reported 0 but nonzero", len(sub0),
              "; C nonzero with B=0", len(cb), "; twins", sum(r["twin"] is not None for r in rs),
              "; unfilled", sum(r["excel"] is None for r in rs))
