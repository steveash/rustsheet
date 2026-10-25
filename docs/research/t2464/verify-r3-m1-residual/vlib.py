"""verify-r3-m1-residual: independent parser, model and scorer for the M2 beta density claim (T-2513).

Written from the RESULTS.md description only (no code from r3-m1-residual is imported):
  tx = fl(fl(a-1) * LN(x)),  ty = fl(fl(b-1) * ly),  ly = log1p_series(-x) (x <= 0.375, r2-log1p) else LN(fl(1-x))
  lnB: min = 1 -> -LN(max); min < 1, max <= 8 -> fl(GL(min) + fl(GL(max) - GL(fl(a+b))));
       min < 1, max > 8 -> fl(GL(min) + algdiv(min, max)), algdiv = TOMS 708 in doubles (own code), Excel LN(b)
  hi = fl(fl(tx+ty) - lnB), lo = (tx+ty-lnB) - hi exactly, density = fl(EXP(hi) * fl(1+lo))
Primitives: Excel's EXP / LN / GAMMALN from ../excelmath.py (engine models); GAMMALN overridden by Excel's measured
values where a probe holds them. Formula arguments are parsed here from the case files: decimal literals are cut
(truncated) to 15 significant digits as Excel does, two-piece literals (m*2^-k+n*2^-j) are evaluated in doubles.
"""

from __future__ import annotations

import csv
import json
import math
import pickle
import re
import struct
import sys
from decimal import ROUND_DOWN, Decimal
from fractions import Fraction
from pathlib import Path

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
REPO = T2464.parents[2]
CASES = REPO / "eval" / "suites" / "probes-numeric" / "cases"
for d in (T2464, T2464 / "r2-log1p"):
    if str(d) not in sys.path:
        sys.path.append(str(d))

import excelmath as em  # noqa: E402
from excel_log1p import log1p_series  # noqa: E402  (r2-log1p's Excel log1p, x87 stored ops)

# ------------------------------------------------------------------------------------------- Excel primitives
_CF = HERE / "cache.pkl"
_C: dict = pickle.loads(_CF.read_bytes()) if _CF.exists() else {}
_MEASURED_GL: dict = {}


def xl(name: str, xs) -> list[float]:
    """Excel's NAME(x) for each x (batched, cached)."""
    need = sorted({x for x in xs if (name, x) not in _C})
    if need:
        for x, v in zip(need, em.excel(name, need)):
            _C[(name, x)] = v
        _CF.write_bytes(pickle.dumps(_C))
    return [_C[(name, x)] for x in xs]


def gl(x: float, measured: bool = True) -> float:
    if measured and x in _MEASURED_GL:
        return _MEASURED_GL[x]
    return _C[("GAMMALN", x)]


# ------------------------------------------------------------------------------------------- formula parsing
TWO = re.compile(r"^\((\d+)\*2\^-(\d+)\+(\d+)\*2\^-(\d+)\)$")
NUM = re.compile(r"^[+-]?(\d+\.?\d*|\.\d+)([Ee][+-]?\d+)?$")
REF = re.compile(r"^\$?([A-Z]+)\$?(\d+)$")


def cut15(s: str) -> float:
    """Excel's reading of a decimal formula literal: truncated to 15 significant digits."""
    d = Decimal(s)
    if d == 0:
        return 0.0
    q = Decimal(1).scaleb(d.adjusted() - 14)
    return float(d.quantize(q, rounding=ROUND_DOWN))


def split_args(s: str) -> list[str]:
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
            continue
        depth += ch == "("
        depth -= ch == ")"
        cur += ch
    out.append(cur.strip())
    return out


def eval_arg(t: str, cells: dict, depth: int = 0):
    t = t.strip()
    if t.upper() in ("TRUE", "FALSE"):
        return t.upper() == "TRUE"
    m = TWO.match(t)
    if m:
        a, k, b, j = (int(g) for g in m.groups())
        return float(a) * 2.0**-k + float(b) * 2.0**-j
    if NUM.match(t):
        return cut15(t)
    m = REF.match(t)
    if m and depth < 4:
        v = cells.get(m.group(1) + m.group(2))
        if isinstance(v, bool):
            return v
        if isinstance(v, (int, float)):
            return float(v)
        if isinstance(v, str) and v.startswith("="):
            return eval_arg(v[1:], cells, depth + 1)
    raise ValueError(f"unparsed argument {t!r}")


CALL = re.compile(r"^=\s*(BETA\.DIST|BETADIST)\((.*)\)$", re.I)


def parse_density_call(f: str, cells: dict):
    """(x, a, b) of =BETA.DIST(x,a,b,FALSE[,0,1]); None if not a plain density call."""
    m = CALL.match(f)
    if not m:
        return None
    args = [eval_arg(t, cells) for t in split_args(m.group(2))]
    if len(args) < 4 or args[3] is not False:
        return None
    if len(args) >= 6 and (args[4], args[5]) != (0.0, 1.0):
        return None
    return tuple(float(v) for v in args[:3])


def load_case(path: Path):
    d = json.loads(path.read_text())
    return {s["name"]: s["cells"] for s in d["sheets"]}


def load_oracle(path: Path):
    d = json.loads(path.read_text())
    return {sn: {k: v.get("v") for k, v in sh.items() if v.get("t") == "n"} for sn, sh in d["sheets"].items()}


# ------------------------------------------------------------------------------------------- datasets
def rows_case(case_id: str, oracle_path: Path | None = None, case_path: Path | None = None, tag: str = ""):
    """Every density cell of a case: dict(x, a, b, excel, src, echo_ok). echo_ok: all argument cells that are
    formulas have an oracle echo equal to the parsed double (None when there is nothing to echo)."""
    cp = case_path or CASES / f"{case_id}.case.json"
    op = oracle_path or CASES / f"{case_id}.oracle.json"
    sheets, orc = load_case(cp), load_oracle(op)
    rows = []
    for sn, cells in sheets.items():
        o = orc.get(sn, {})
        for cell, f in cells.items():
            if not isinstance(f, str):
                continue
            p = parse_density_call(f, cells)
            if p is None or cell not in o:
                continue
            echo = None
            for t in split_args(CALL.match(f).group(2))[:3]:
                m = REF.match(t)
                if m and isinstance(cells.get(m.group(1) + m.group(2)), str):
                    ev = o.get(m.group(1) + m.group(2))
                    ok = ev == eval_arg(t, cells)
                    echo = ok if echo is None else (echo and ok)
            rows.append(
                dict(x=p[0], a=p[1], b=p[2], excel=o[cell], src=f"{tag or case_id}!{sn}!{cell}", echo_ok=echo)
            )
    return rows


def rows_old():
    """The old pdf cells (beta_exact.tsv's case/cell list), re-parsed here from the case files."""
    want: dict = {}
    for r in csv.DictReader(open(T2464 / "beta_exact.tsv"), delimiter="\t"):
        if r["cum"] == "0":
            cid = r["case"].split("/")[-1].replace(".case.json", "")
            want.setdefault(cid, {})[r["cell"]] = (float(r["x"]), float(r["a"]), float(r["b"]))
    out = []
    for cid, cells in want.items():
        for r in rows_case(cid):
            sn, cell = r["src"].split("!")[1:]
            key = f"{sn}!{cell}"
            if key in cells:
                r["tsv_args"] = cells[key]
                out.append(r)
    return out


def rows_probe1():
    p = T2464 / "w1-density" / "probe"
    return rows_case(
        "probe1", oracle_path=p / "probe1.out.json", case_path=p / "probe1.case.json", tag="probe1"
    )


def load_measured_gl(case_id: str) -> int:
    """Excel's GAMMALN measured in a case: cells =GAMMALN(<arg>) with a numeric oracle value."""
    sheets, orc = load_case(CASES / f"{case_id}.case.json"), load_oracle(CASES / f"{case_id}.oracle.json")
    n = 0
    for sn, cells in sheets.items():
        for cell, f in cells.items():
            m = re.match(r"^=GAMMALN\((.*)\)$", f) if isinstance(f, str) else None
            if m and cell in orc.get(sn, {}):
                arg = eval_arg(m.group(1), cells)
                if REF.match(m.group(1).strip()):
                    ref = REF.match(m.group(1).strip())
                    rc = ref.group(1) + ref.group(2)
                    if isinstance(cells.get(rc), str) and orc[sn].get(rc) != arg:
                        continue  # echo mismatch: argument not what we think
                _MEASURED_GL[arg] = orc[sn][cell]
                n += 1
    return n


# ------------------------------------------------------------------------------------------- model M2
C0, C1, C2, C3, C4, C5 = (
    0.0833333333333333,
    -0.00277777777760991,
    7.9365066682539e-4,
    -5.9520293135187e-4,
    8.37308034031215e-4,
    -0.00165322962780713,
)
P1, P2, P3 = -1.29418923021993, 0.405303492862024, -0.0178874546012214
Q1, Q2, Q3 = -1.62752256355323, 0.747811014037616, -0.0845104217945565


def alnrel(a: float) -> float:
    """TOMS 708 alnrel for |a| <= 0.375, doubles."""
    t = a / (a + 2.0)
    t2 = t * t
    w = (((P3 * t2 + P2) * t2 + P1) * t2 + 1.0) / (((Q3 * t2 + Q2) * t2 + Q1) * t2 + 1.0)
    return 2.0 * t * w


def algdiv(a: float, b: float, lnb: float) -> float:
    """TOMS 708 algdiv(a, b) = ln(Gamma(b)/Gamma(a+b)), b >= 8, every op a double op; lnb = LN(b)."""
    h = b / a if a > b else a / b
    hp = 1.0 + h
    c, x = (1.0 / hp, h / hp) if a > b else (h / hp, 1.0 / hp)
    d = a + (b - 0.5) if a > b else b + (a - 0.5)
    x2 = x * x
    s = [1.0 + (x + x2)]  # s3, s5, ..., s11: s_{k+2} = 1 + (x + x2 * s_k)
    for _ in range(4):
        s.append(1.0 + (x + x2 * s[-1]))
    s3, s5, s7, s9, s11 = s
    t = (1.0 / b) ** 2
    w = ((((C5 * s11 * t + C4 * s9) * t + C3 * s7) * t + C2 * s5) * t + C1 * s3) * t + C0
    w = w * (c / b)
    u = d * alnrel(a / b)
    v = a * (lnb - 1.0)
    return (w - v) - u if u > v else (w - u) - v


def _round_bits(q: Fraction, bits: int) -> Fraction:
    """q rounded to `bits` significant bits, ties to even (exact rational arithmetic)."""
    if q == 0:
        return q
    s = -1 if q < 0 else 1
    q = abs(q)
    e = q.numerator.bit_length() - q.denominator.bit_length()
    if Fraction(2) ** e > q:
        e -= 1
    scale = Fraction(2) ** (bits - 1 - e)
    m = q * scale
    f = m.numerator // m.denominator
    r = m - f
    if r > Fraction(1, 2) or (r == Fraction(1, 2) and f % 2):
        f += 1
    return s * f / scale


def x87(op: str, a: float, b: float) -> float:
    """A stored x87 operation: the exact result rounded to 64 bits, then to a double (double rounding)."""
    A, B = Fraction(a), Fraction(b)
    r = {"+": A + B, "-": A - B, "*": A * B}[op]
    return float(_round_bits(r, 64))


ARITH = {"d": lambda op, a, b: {"+": a + b, "-": a - b, "*": a * b}[op], "x": x87}


def region(a: float, b: float) -> str:
    lo, hi = min(a, b), max(a, b)
    if lo == 1.0:
        return "min1"
    return "gl" if hi <= 8.0 else "algdiv"


def prefetch(rows) -> None:
    lnx, gls = set(), set()
    for r in rows:
        x, a, b = r["x"], r["a"], r["b"]
        lnx.add(x)
        if x > 0.375:
            lnx.add(1.0 - x)
        lo, hi = min(a, b), max(a, b)
        lnx.add(hi)
        gls.update((lo, hi, a + b))
    xl("LN", list(lnx))
    xl("GAMMALN", list(gls))


def lnb_m2(a: float, b: float, measured: bool = True, alg: str = "toms", ar: str = "d") -> float:
    """lnB of M2; ar = "d" double ops or "x" stored x87 ops for the GL sums (algdiv itself stays in doubles)."""
    op = ARITH[ar]
    lo, hi = min(a, b), max(a, b)
    if lo == 1.0:
        return -_C[("LN", hi)]
    if hi <= 8.0:
        return op("+", gl(lo, measured), op("-", gl(hi, measured), gl(op("+", a, b), measured)))
    if alg == "exact":
        import mpmath

        mpmath.mp.prec = 200
        al = mpmath.loggamma(mpmath.mpf(hi)) - mpmath.loggamma(mpmath.mpf(lo) + mpmath.mpf(hi))
        return float(mpmath.mpf(gl(lo, measured)) + al)
    return op("+", gl(lo, measured), algdiv(lo, hi, _C[("LN", hi)]))


def exponent(r, lnb: float, ar: str = "d"):
    """(hi, fl(1 + lo)); ar = "d" or "x" for the products tx, ty and a-1, b-1 (hi is always summed in doubles)."""
    op = ARITH[ar]
    x, a, b = r["x"], r["a"], r["b"]
    ly = log1p_series(-x) if x <= 0.375 else _C[("LN", 1.0 - x)]
    tx = op("*", op("-", a, 1.0), _C[("LN", x)])
    ty = op("*", op("-", b, 1.0), ly)
    hi = (tx + ty) - lnb
    lo = (Fraction(tx) + Fraction(ty) - Fraction(lnb)) - Fraction(hi)
    return hi, float(1 + lo)


def m2(rows, lnb_of, ar: str = "d") -> list[float]:
    """Model M2 densities for rows, lnB given per shape by lnb_of(a, b)."""
    prefetch(rows)
    cache: dict = {}
    parts = []
    for r in rows:
        k = (r["a"], r["b"])
        if k not in cache:
            cache[k] = lnb_of(*k)
        parts.append(exponent(r, cache[k], ar))
    ex = xl("EXP", [h for h, _ in parts])
    return [e * f for e, (_, f) in zip(ex, parts)]


# ------------------------------------------------------------------------------------------- scoring
def _key(v: float) -> int:
    i = struct.unpack("<q", struct.pack("<d", v))[0]
    return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)


def ulpdist(m: float, e: float) -> float:
    if not (math.isfinite(m) and math.isfinite(e)):
        return math.inf
    return abs(_key(m) - _key(e))


def within15(m: float, e: float) -> bool:
    if not (math.isfinite(m) and math.isfinite(e)):
        return False
    if e == 0:
        return m == 0
    half = Decimal(5).scaleb(Decimal(repr(abs(e))).adjusted() - 15)
    return abs(Decimal(m) - Decimal(e)) <= half


def score(models, excel) -> tuple[int, int, int, int]:
    n = ex = w4 = w15 = 0
    for m, e in zip(models, excel):
        n += 1
        ex += m == e
        w4 += ulpdist(m, e) <= 4
        w15 += within15(m, e)
    return n, ex, w4, w15


def fmt(s) -> str:
    return f"{s[1]:5d} / {s[2]:5d} / {s[3]:5d} of {s[0]:5d}"
