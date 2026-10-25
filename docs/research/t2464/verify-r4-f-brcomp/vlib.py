"""T-2586 verify-r4-f-brcomp: an independent parser, exact reference, scorer and re-implementation of r4-f-brcomp's
frozen M-xd F density model, written from its RESULTS.md description (not from its code).

M-xd (as described): a = m/2, b = n/2, h = a + b, mx = fl(m*x) (x87 product stored), side 1 if mx <= n:
  r = fl(mx/n), c = a; side 2: r = fl(n/mx), c = -b.  L = Excel log1p(r): r2-log1p's 14-term atanh series for
  r <= 0.375, else LN(fl(1+r)).  K = exact(-lnB(a,b) + c*ln(m/n)) rounded to 64 bits.
  E = RN64(RN64(K + fl(c*LN(x))) - RN64(h*L)),  F = fl(EXPV6(E) / x).
Excel's LN comes from excelmath (the engine's bit-exact model); EXP V6 on an extended argument is re-implemented
here from x87_exp.rs's module docs. Run with /home/user/rustsheet/bindings/python/.venv/bin/python."""

from __future__ import annotations

import json
import math
import re
import sys
from decimal import Decimal
from fractions import Fraction
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
T = HERE.parent
REPO = T.parent.parent.parent
CASES = REPO / "eval" / "suites" / "probes-numeric" / "cases"
sys.path.insert(0, str(T))
import excelmath as XM  # noqa: E402  (Excel's LN, bit-exact engine model)

mpf = mpmath.mpf
mpmath.mp.prec = 200


# ------------------------------------------------------------------ rounding
def rn(v, bits: int) -> mpf:
    """Round-to-nearest-even of an exact mpf to `bits` significant bits."""
    with mpmath.workprec(bits):
        return +mpf(v)


def r64(v) -> mpf:
    return rn(v, 64)


def r53(v) -> float:
    """Stored x87 result: RN53(RN64(v)) (double rounding through the 64-bit register)."""
    return float(rn(r64(v), 53))


def exact(op: str, a, b) -> mpf:
    with mpmath.workprec(400):
        A, B = mpf(a), mpf(b)
        if op == "+":
            return A + B
        if op == "-":
            return A - B
        if op == "*":
            return A * B
        return A / B


# ------------------------------------------------------------------ Excel EXP V6 on an extended argument
L2E64 = mpf(0xB8AA3B295C17F0BC) * mpf(2) ** -63


def exp_v6(E) -> float:
    t = r64(exact("*", E, L2E64))
    with mpmath.workprec(400):
        fl_ = int(mpmath.floor(t))
        rest = t - fl_
        n = fl_ + (1 if rest > 0.5 or (rest == 0.5 and fl_ % 2) else 0)  # FRNDINT, ties to even
        f = t - n
    if f == 0:
        w = mpf(1)
    else:
        with mpmath.workprec(400):
            u = r64(mpmath.power(2, abs(f)) - 1)
        w = r64(exact("+", 1, u))
        if f < 0:
            w = r64(exact("/", 1, w))
    return float(rn(w * mpf(2) ** n, 53))


# ------------------------------------------------------------------ Excel log1p series (r2-log1p's form)
CS = [1.0 / (2 * k + 1) for k in range(15)]


def log1p_series(a: float) -> float:
    r = r53(exact("/", a, a + 2.0))
    t2 = r53(exact("*", r, r))
    p, s = r, None
    for k in range(1, 15):
        p = r53(exact("*", p, t2))
        tm = r53(exact("*", p, CS[k]))
        s = tm if s is None else r53(exact("+", s, tm))
    return 2 * r53(exact("+", r, s))


_LN: dict = {}


def ln_batch(xs) -> None:
    need = sorted({v for v in xs if v not in _LN})
    for i in range(0, len(need), 20000):
        chunk = need[i : i + 20000]
        for v, y in zip(chunk, XM.excel("LN", chunk)):
            _LN[v] = y


# ------------------------------------------------------------------ model
def _pre(x: float, m: float, n: float):
    mx = r53(exact("*", m, x))
    if mx <= n:
        return 1, mx, r53(exact("/", mx, n))
    return 2, mx, r53(exact("/", n, mx))


def lnB(a, b) -> mpf:
    return mpmath.loggamma(a) + mpmath.loggamma(b) - mpmath.loggamma(mpf(a) + b)


def mxd(rows) -> list[float]:
    """M-xd for each row dict (x, d1, d2), Excel LN fetched in two batches."""
    pre = [_pre(r["x"], r["d1"], r["d2"]) for r in rows]
    ss = [r53(exact("+", 1, rr)) for (_, _, rr) in pre if rr > 0.375]
    ln_batch([r["x"] for r in rows] + ss)
    out = []
    for r, (side, _mx, rr) in zip(rows, pre):
        x, m, n = r["x"], r["d1"], r["d2"]
        a, b = m / 2, n / 2
        h = a + b
        c = a if side == 1 else -b
        L = log1p_series(rr) if rr <= 0.375 else _LN[r53(exact("+", 1, rr))]
        hl = r64(exact("*", h, L))
        P = r53(exact("*", c, _LN[x]))
        with mpmath.workprec(300):
            Kx = -lnB(mpf(a), mpf(b)) + c * (mpmath.log(m) - mpmath.log(n))
        K = r64(Kx)
        E = r64(exact("-", r64(exact("+", K, P)), hl))
        v = exp_v6(E)
        out.append(r53(exact("/", v, x)))
    return out


def f_exact(x: float, m: float, n: float) -> mpf:
    with mpmath.workprec(250):
        X, M, N = mpf(x), mpf(m), mpf(n)
        a, b = M / 2, N / 2
        lf = a * mpmath.log(M / N) + (a - 1) * mpmath.log(X) - (a + b) * mpmath.log1p(M * X / N) - lnB(a, b)
        return mpmath.exp(lf)


# ------------------------------------------------------------------ scoring
def w15(model: float, excel: float) -> bool:
    if model == excel:
        return True
    if not math.isfinite(model) or excel == 0:
        return False
    d = Decimal(excel)
    e = d.adjusted()  # exponent of the leading digit (exact on the decimal expansion of the double)
    return abs(Fraction(model) - Fraction(excel)) <= Fraction(1, 2) * Fraction(10) ** (e - 14)


def tally(pairs):
    n = ex = u4 = d15 = 0
    for mod, xl in pairs:
        n += 1
        ex += mod == xl
        u4 += math.isfinite(mod) and abs(mod - xl) <= 4 * math.ulp(xl)
        d15 += w15(mod, xl)
    return n, ex, u4, d15


def fmt(t) -> str:
    return f"{t[1]}/{t[2]}/{t[3]} of {t[0]}"


# ------------------------------------------------------------------ parsing
_EXACT = re.compile(r"^=?\(?\s*(-?\d+)\s*\*\s*2\^(-?\d+)\s*(?:\+\s*(-?\d+)\s*\*\s*2\^(-?\d+)\s*)?\)?$")
_FD = re.compile(r"^=F\.DIST\(\s*([^,]+?)\s*,\s*([^,]+?)\s*,\s*([^,]+?)\s*,\s*(FALSE|0)\s*\)$", re.I)


def lit15(s: str) -> float:
    """A numeric formula literal as Excel reads it: truncated (not rounded) to 15 significant digits."""
    d = Decimal(s)
    if d == 0:
        return 0.0
    sign, digits, exp = d.as_tuple()
    if len(digits) > 15:
        cut = len(digits) - 15
        d = Decimal((sign, digits[:15], exp + cut))
    return float(d)


def exact_expr(s: str) -> float | None:
    m = _EXACT.match(s.strip())
    if not m:
        return None
    h, e1, lo, e2 = (int(g) if g is not None else 0 for g in m.groups())
    v = Fraction(h) * Fraction(2) ** e1 + Fraction(lo) * Fraction(2) ** e2
    f = float(v)
    assert Fraction(f) == v, s  # the encoder's sum is an exact double
    return f


def arg_value(s: str, cells: dict) -> float | None:
    if re.fullmatch(r"[A-Z]+\d+", s):
        f = cells.get(s)
        if not isinstance(f, str):
            return None if f is None else float(f)
        v = exact_expr(f)
        if v is None and re.fullmatch(r"=?-?[\d.]+(E[-+]?\d+)?", f, re.I):
            v = lit15(f.lstrip("="))
        return v
    v = exact_expr(s)
    if v is not None:
        return v
    if re.fullmatch(r"-?[\d.]+(E[-+]?\d+)?", s, re.I):
        return lit15(s)
    return None


def case_rows(case: str):
    """F.DIST(.,FALSE) cells of a probes-numeric case with Excel's oracle value; x echo checked when present."""
    cj = json.loads((CASES / f"{case}.case.json").read_text())
    oj = json.loads((CASES / f"{case}.oracle.json").read_text())
    cells = cj["sheets"][0]["cells"]
    ov = oj["sheets"]["Calc"]
    rows, bad_echo = [], 0
    for ref, f in cells.items():
        if not isinstance(f, str):
            continue
        m = _FD.match(f)
        if not m:
            continue
        xs, d1s, d2s, _ = m.groups()
        x = arg_value(xs, cells)
        d1, d2 = arg_value(d1s, cells), arg_value(d2s, cells)
        o = ov.get(ref)
        if x is None or d1 is None or d2 is None or not o or o.get("t") != "n":
            continue
        if re.fullmatch(r"[A-Z]+\d+", xs):
            echo = ov.get(xs, {}).get("v")
            if echo != x:
                bad_echo += 1
                continue
        v = float(o["v"])
        if not (x > 0 and v > 0):
            continue
        rows.append(
            dict(src=case, cell=ref, x=x, d1=float(math.trunc(d1)), d2=float(math.trunc(d2)), excel=v)
        )
    return rows, bad_echo


def tsv_rows(flags=("False",)):
    out = []
    for line in (T / "cells.tsv").read_text().splitlines()[1:]:
        fn, case, cell, args, xl = line.split("\t")
        if fn != "F.DIST":
            continue
        a = args.split("|")
        if len(a) != 4 or a[3] not in flags:
            continue
        try:
            x, d1, d2 = (lit15(s) for s in a[:3])
            v = float(xl)
        except Exception:  # noqa: BLE001  (error values such as '#NUM!')
            continue
        if not (x > 0 and v > 0 and math.isfinite(v)) or d1 < 1 or d2 < 1:
            continue
        out.append(
            dict(
                src="cells",
                cell=f"{case}!{cell}",
                x=x,
                d1=float(math.trunc(d1)),
                d2=float(math.trunc(d2)),
                excel=v,
            )
        )
    return out


def cls(r) -> str:
    if max(r["d1"], r["d2"]) >= 50:
        return "max>=50"
    if min(r["d1"], r["d2"]) <= 2:
        return "min<=2"
    return "other"
