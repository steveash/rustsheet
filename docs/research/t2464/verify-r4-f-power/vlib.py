"""verify-r4-f-power: own loaders, exact F density, scorer and an independent implementation of the r4 `dd`/`ddl`
models (written from the RESULTS.md description; nothing imported from r4-f-power).

Shared, already-verified pieces used: excelmath (Excel LN by the engine's bit-exact model; fl53/fl64 rounding) and
r2-log1p's excel_log1p.log1p_series (Excel's log1p, statistics.md "14-term atanh series").
"""

from __future__ import annotations

import json
import math
import re
import sys
from decimal import Decimal
from functools import lru_cache
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
T = HERE.parent
ROOT = T.parents[2]
CASES = ROOT / "eval/suites"
for p in (T, T / "r2-log1p"):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))

import excelmath as XM  # noqa: E402
from excel_log1p import log1p_series  # noqa: E402

mpf = mpmath.mpf
mpmath.mp.prec = 300
fl53, fl64 = XM.fl53, XM.fl64


# ------------------------------------------------------------------ parsing
def lit15(s: str) -> float:
    """A formula number literal as Excel reads it: digits past the 15th significant one are truncated."""
    d = Decimal(s)
    sign, digits, exp = d.as_tuple()
    digits = tuple(digits)
    while len(digits) > 1 and digits[0] == 0:
        digits = digits[1:]
    if len(digits) > 15:
        exp += len(digits) - 15
        digits = digits[:15]
    return float(Decimal((sign, digits, exp)))


ENC = re.compile(r"^=\((\d+)\*2\^(-?\d+)\+(\d+)\*2\^(-?\d+)\)$")


def enc_value(f: str) -> float:
    """The exact value of a two-piece encoder formula =(p*2^e+q*2^k), rounded once to a double."""
    m = ENC.match(f)
    if not m:
        raise ValueError(f)
    p, e, q, k = (int(g) for g in m.groups())
    return fl53(mpf(p) * mpf(2) ** e + mpf(q) * mpf(2) ** k)


def split_args(s: str) -> list[str]:
    """Top-level comma split of a call's argument text, each part stripped."""
    parts, depth, start = [], 0, 0
    for pos, ch in enumerate(s):
        depth += (ch == "(") - (ch == ")")
        if ch == "," and depth == 0:
            parts.append(s[start:pos].strip())
            start = pos + 1
    parts.append(s[start:].strip())
    return parts


def load_case(case_rel: str):
    """(cells, oracle values) for a case path relative to eval/suites (e.g. 'probes-numeric/x.case.json')."""
    suite, rest = case_rel.split("/", 1)
    p = CASES / suite / "cases" / rest
    case = json.loads(p.read_text())
    side = json.loads(p.with_name(p.name.replace(".case.json", ".oracle.json")).read_text())
    sheets = {s["name"]: s["cells"] for s in case["sheets"]}
    return sheets, side["sheets"]


def oval(side, sheet, ref):
    c = side.get(sheet, {}).get(ref)
    if not c or c.get("t") != "n":
        return None
    return float(c["v"])


FD = re.compile(r"^=(?:_xlfn\.)?F\.DIST\((.*)\)$", re.I)
NUM = re.compile(r"^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$")
REF = re.compile(r"^(?:(\w+)!)?\$?([A-Z]+)\$?(\d+)$")


def resolve(arg: str, sheet: str, sheets, side):
    """A numeric/bool F.DIST argument: literal (15-digit truncation), TRUE/FALSE, or a reference (Excel's value; an
    echo of an encoder formula is checked against the formula's exact value)."""
    a = arg.strip()
    if a.upper() in ("TRUE", "FALSE"):
        return a.upper() == "TRUE"
    neg = a.startswith("-")
    body = a[1:] if neg else a
    if NUM.match(body):
        v = lit15(body)
        return -v if neg else v
    m = REF.match(a)
    if m:
        sh = m.group(1) or sheet
        ref = m.group(2) + m.group(3)
        v = oval(side, sh, ref)
        f = sheets.get(sh, {}).get(ref)
        if isinstance(f, str) and ENC.match(f) and v is not None:
            if enc_value(f) != v:
                raise ValueError(f"echo misread {sh}!{ref}")
        if v is None and isinstance(f, (int, float)):
            v = float(f)
        if v is None and isinstance(f, str) and NUM.match(f):
            v = lit15(f)
        return v
    return None


def fdist_rows(case_rel: str, tag: str = ""):
    """All F.DIST(x, d1, d2, cum=FALSE) cells of a case with a positive Excel value: dicts x, d1, d2, excel, key."""
    sheets, side = load_case(case_rel)
    out = []
    for sh, cells in sheets.items():
        for ref, f in cells.items():
            if not isinstance(f, str):
                continue
            m = FD.match(f)
            if not m:
                continue
            args = split_args(m.group(1))
            if len(args) != 4:
                continue
            vals = [resolve(a, sh, sheets, side) for a in args]
            if any(v is None for v in vals):
                continue
            x, d1, d2, cum = vals
            if bool(cum):
                continue
            ev = oval(side, sh, ref)
            if ev is None or not ev > 0 or not math.isfinite(ev):
                continue
            out.append(
                dict(
                    x=float(x),
                    d1=float(math.trunc(d1)),
                    d2=float(math.trunc(d2)),
                    excel=ev,
                    key=f"{case_rel}:{sh}!{ref}",
                    tag=tag,
                )
            )
    return out


def cells_tsv_cases():
    """Case paths of the F.DIST pdf rows in cells.tsv (re-parsed from the cases themselves)."""
    seen = []
    for line in (T / "cells.tsv").read_text().splitlines()[1:]:
        fn, case = line.split("\t")[:2]
        if fn == "F.DIST" and case not in seen:
            seen.append(case)
    return seen


# ------------------------------------------------------------------ exact density and scoring
@lru_cache(maxsize=None)
def kprime(m: float, n: float, side1: bool):
    a, b, h = mpf(m) / 2, mpf(n) / 2, (mpf(m) + mpf(n)) / 2
    k = mpmath.loggamma(h) - mpmath.loggamma(a) - mpmath.loggamma(b)
    return k + (a if side1 else -b) * mpmath.log(mpf(m) / mpf(n))


def exact(x: float, m: float, n: float):
    """f(x) = exp(lnB^-1 + a ln(m/n) + (a-1) ln x - h ln(1 + m x / n)), at 300 bits."""
    X, M, N = mpf(x), mpf(m), mpf(n)
    a, h = M / 2, (M + N) / 2
    return mpmath.exp(kprime(m, n, True) + (a - 1) * mpmath.log(X) - h * mpmath.log1p(M * X / N))


def within15(v: float, e: float) -> bool:
    if v == e:
        return True
    if not math.isfinite(v):
        return False
    q = math.floor(math.log10(abs(e))) - 14
    return abs(Decimal(v) - Decimal(e)) <= Decimal(5) * Decimal(10) ** (q - 1)


def tally(pairs):
    """(exact, within 4 ulp, within 15 digits, n) of (model, excel) pairs."""
    ex = w4 = w15 = 0
    for v, e in pairs:
        ex += v == e
        w4 += math.isfinite(v) and abs(v - e) <= 4 * math.ulp(e)
        w15 += within15(v, e)
    return ex, w4, w15, len(pairs)


# ------------------------------------------------------------------ Excel pieces
_LN: dict = {}


def ln_batch(vs):
    need = sorted({float(v) for v in vs if float(v) not in _LN})
    if need:
        for v, r in zip(need, XM.excel("LN", need)):
            _LN[v] = r


def LN(v: float) -> float:
    v = float(v)
    if v not in _LN:
        ln_batch([v])
    return _LN[v]


def x87(op, a, b) -> float:
    """fl53(fl64(a op b)); excelmath.x87 evaluates all four ops eagerly and so fails on a * 0."""
    A, B = mpf(a), mpf(b)
    r = A + B if op == "+" else A - B if op == "-" else A * B if op == "*" else A / B
    return fl53(fl64(r))


LOG2E64 = fl64(1 / mpmath.log(2))


def exp_v6(E) -> mpmath.mpf:
    """Excel's EXP (model V6, statistics.md/x87_exp.rs) on an extended (64-bit) argument, result kept extended:
    t = fl64(E * fl64(log2 e)), n = nint(t), f = t - n, u = fl64(2^|f| - 1), w = fl64(1 + u), w = fl64(1/w) if f < 0."""
    E = fl64(E)
    t = fl64(E * LOG2E64)
    n = int(mpmath.nint(t))
    f = t - n
    u = fl64(mpf(2) ** abs(f) - 1)
    w = fl64(1 + u)
    if f < 0:
        w = fl64(1 / w)
    return mpmath.ldexp(w, n)


def log1p_xl(r: float) -> float:
    """Excel's log1p: the series for |r| <= 0.375, else LN(fl(1 + r)) (r3/r4's switch; the positive side is the
    worker's assumption)."""
    if -0.375 <= r <= 0.375:
        return log1p_series(r)
    return LN(x87("+", 1.0, r))


def model(x: float, m: float, n: float, var: str) -> float:
    """r4's frozen family, implemented from its description.
    dd : F = fl(fl64(EXP_V6(fl64(fl(fl(K_hi + H) + P) + K_lo)) / x))
    ddl: F = fl(EXP_V6(fl64(S2 + K_lo - LN(x))))
    with K_hi = fl(K'), K_lo = K' - K_hi, P = +-fl(c LN(x)), H = -fl64(h log1p_xl(r)), sums x87 stored."""
    a, b, h = m / 2, n / 2, (m + n) / 2
    mx = x87("*", m, x)
    side1 = mx <= n
    r = x87("/", mx, n) if side1 else x87("/", n, mx)
    L = LN(x)
    c = mpf(a) if side1 else -mpf(b)
    if var == "dd-P64":  # ablation: the product kept extended
        P = fl64(c * L)
    elif var == "dd-Pex":  # ablation: the exact c ln x, one rounding to double
        P = fl53(c * mpmath.log(mpf(x)))
    else:
        P = x87("*", a, L) if side1 else -x87("*", b, L)
    H = -fl64(mpf(h) * log1p_xl(r))
    Kp = kprime(m, n, side1)
    Khi = fl53(Kp)
    Klo = Kp - Khi
    S1 = fl64(Khi + H) if var == "dd-S1x" else fl53(fl64(Khi + H))  # ablation: first sum kept extended
    S2 = fl53(fl64(S1 + P))
    if var == "dd-nolo":  # ablation: no double-double constant (K_lo dropped)
        Klo = 0
    if var.startswith("dd-") or var == "dd":
        return fl53(fl64(exp_v6(fl64(S2 + Klo)) / x))
    if var == "ddl":
        return fl53(exp_v6(fl64(S2 + Klo - L)))
    if var == "m1":  # r3's ratio model with the stored P: E = K' + fl64(fl64(P - LN(x)) + H), no stored sums
        return fl53(exp_v6(fl64(Kp + fl64(fl64(mpf(P) - L) + H))))
    if var == "CR":
        return fl53(exact(x, m, n))
    raise ValueError(var)
