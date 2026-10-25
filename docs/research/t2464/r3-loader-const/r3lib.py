"""T-2513 r3-loader-const: shared helpers for pinning the shape constant of Excel's Loader beta density (a, b > 1).

- Probe design (deterministic, seeded): x values where BOTH bd0 sides take their log form (v >= 0.205 on each side,
  Excel's cut is in (0.1837, 0.2000001]), so that r2-density-large's x87 X part is the modelled one.
- Probe rows are rebuilt from the generator (no meta files); Excel values come from the eval sidecars.
- E-set inversion of Excel's EXP (model V6) and per-cell constraints on the shape constant.
Reuses r2-density-large (dlib, loader, model) and w2-forensics' exact-double encoder by import.
"""

from __future__ import annotations

import json
import math
import pickle
import random
import sys
from pathlib import Path

HERE = Path(__file__).parent
T = HERE.parent
for p in (T, T / "r2-density-large", T / "w2-forensics"):
    if str(p) not in sys.path:
        sys.path.append(str(p))

import mpmath  # noqa: E402

import dlib as L  # noqa: E402  (r2-density-large)
import loader as LD  # noqa: E402
import make_probes as MP  # noqa: E402  (w2-forensics encoder)
import model as M2  # noqa: E402

mpf = mpmath.mpf
CASES = MP.CASES
PREFIX = "t2513-r3-loader-const"
BEST_X = M2.BEST_X
CUTV = 0.205

# ------------------------------------------------------------------ shapes
def _pairs(spec: str):
    """'a,b a,b ...' -> [(a, b), ...] (kept as one string so the shape lists stay compact)."""
    return [tuple(float(v) for v in p.split(",")) for p in spec.split()]


# k, n-k not half-integers; then half-integer controls, integer controls, and min(a,b) < 2
SWEEP_BASES = _pairs(
    "5.3,9.7 3.7,2.9 20.3,30.1 12.7,40.3 2.3,4.6 4.2,3.3 6.7,6.1 9.4,15.8 3.15,7.85 14.6,9.2 30.7,25.3 45.2,60.9 "
    "7.3,11.6 2.5,3.5 4.5,7.5 7.5,11.5 4,4 6,9 12,20 1.6,5.3 7.3,1.45 1.3,1.8"
)
HOLD_BASES = [(8.3, 5.9), (2.7, 6.4), (17.1, 11.3)]


def nshift(v: float, k: int) -> float:
    return MP.step(v, k)


def shape_pieces(a: float, b: float):
    """k = a-1, n = fl(a+b)-2, nk = n-k as Excel's x87 stored ops (exact mpf of the doubles)."""
    k = L.D(mpf(a) - 1)
    n = L.D(mpf(L.D(mpf(a) + mpf(b))) - 2)
    nk = L.D(mpf(n) - mpf(k))
    return k, n, nk


def vsides(x: float, a: float, b: float):
    k, n, nk = (float(v) for v in shape_pieces(a, b))
    q = 1.0 - x
    np_, nq = n * x, n * q
    return abs(k - np_) / (k + np_), abs(nk - nq) / (nk + nq)


def both_log_xs(a: float, b: float, rng: random.Random, N: int, vmax: float = 0.42):
    """N random doubles x in (0, 1) with both bd0 sides at v in [CUTV, vmax] (smallest |ln pdf| region)."""
    import numpy as np

    k, n, nk = (float(v) for v in shape_pieces(a, b))
    g = np.arange(1, 20000) / 20000
    vp = np.abs(k - n * g) / (k + n * g)
    vq = np.abs(nk - n * (1 - g)) / (nk + n * (1 - g))
    vm = np.minimum(vp, vq)
    ok = list(g[(vm >= CUTV + 0.002) & (vm <= CUTV + 0.04) & (np.maximum(vp, vq) <= vmax)])
    if not ok:
        ok = list(g[vm >= CUTV + 0.002])
        ok = ok[:200] + ok[-200:]
    lp = [approx_lnpdf(a, b, x) for x in ok]  # keep the high-density tail only (small |E|: a fine pin)
    ok = [x for x, v in zip(ok, lp) if v >= max(lp) - 3]
    out: list[float] = []
    while len(out) < N:
        x = float(rng.choice(ok) + rng.uniform(-2.5e-5, 2.5e-5))
        if 0 < x < 1 and min(vsides(x, a, b)) >= CUTV and x not in out:
            out.append(x)
    return out


def approx_lnpdf(a: float, b: float, x: float) -> float:
    return (a - 1) * math.log(x) + (b - 1) * math.log1p(-x) - (math.lgamma(a) + math.lgamma(b) - math.lgamma(a + b))


def random_shapes(rng: random.Random, count: int, lo=1.05, hi=80.0):
    out = []
    while len(out) < count:
        a = round(math.exp(rng.uniform(math.log(lo), math.log(hi))), rng.choice((1, 2, 3)))
        b = round(math.exp(rng.uniform(math.log(lo), math.log(hi))), rng.choice((1, 2, 3)))
        if a > 1 and b > 1 and (a, b) not in out and approx_lnpdf(a, b, both_log_xs(a, b, rng, 1)[0]) > -12:
            out.append((a, b))
    return out


def design(which: str):
    """Rows (blk, x, a, b) of probe `which`: 'sweep', 'diverse', 'grid' (in-sample) or 'hold' (hold-out)."""
    rows = []
    if which in ("sweep", "hold"):
        bases = SWEEP_BASES if which == "sweep" else HOLD_BASES
        rng = random.Random(25130 if which == "sweep" else 25139)
        for a0, b0 in bases:
            pts = [("a", nshift(a0, i), b0) for i in range(-8, 8)] + [
                ("b", a0, nshift(b0, i)) for i in range(-8, 8)
            ]
            for i, (w, a, b) in enumerate(pts):
                for x in both_log_xs(a, b, rng, 16):
                    rows.append((f"sw({a0:g},{b0:g}){w}{i % 16 - 8:+d}", x, a, b))
    if which == "grid":  # every half-integer shape pair: k, n-k (and n <= 15) on R's stirlerr table
        rng = random.Random(25133)
        halves = [h / 2 for h in range(3, 33)]
        for a in halves:
            for b in halves:
                for x in both_log_xs(a, b, rng, 8):
                    rows.append((f"gr({a:g},{b:g})", x, a, b))
    if which in ("diverse", "hold"):
        rng = random.Random(25131 if which == "diverse" else 25137)
        shapes = random_shapes(rng, 400 if which == "diverse" else 110)
        if which == "diverse":
            shapes += _pairs(
                "2.5,2.5 1.5,1.5 3.5,2.5 40,1.03 1.03,40 100,120 30.7,1.7 11.6,7.3 1.9,3.3 8,12 8,40 3,2 5,9 10,10"
            )
        for a, b in shapes:
            for x in both_log_xs(a, b, rng, 24 if which == "diverse" else 16):
                rows.append((f"sh({a:g},{b:g})", x, a, b))
    return rows


def case_name(which: str, part: int) -> str:
    return f"{PREFIX}-{which}{part}"


def split(rows, per=14000):
    return [rows[i : i + per] for i in range(0, len(rows), per)]


def build_cases(which: str):
    """Case dicts: column A the density (literal formula), B/C/D echo x/a/b on some rows (parser check; the grid
    probe echoes only its shapes, to stay inside the cell budget)."""
    out = {}
    for part, chunk in enumerate(split(design(which)), 1):
        cells = {}
        seen = set()
        for i, (_blk, x, a, b) in enumerate(chunk, 1):
            cells[f"A{i}"] = f"=BETA.DIST({MP.num(x)},{MP.num(a)},{MP.num(b)},FALSE)"
            if i % 8 == 1 and which != "grid":
                cells[f"B{i}"] = f"={MP.num(x)}"
            if (a, b) not in seen:
                seen.add((a, b))
                cells[f"C{i}"] = f"={MP.num(a)}"
                cells[f"D{i}"] = f"={MP.num(b)}"
        assert len(cells) <= 16384, len(cells)
        name = case_name(which, part)
        out[name] = {
            "format": "rustsheet-case/v1",
            "description": f"T-2513 r3-loader-const ({which}): BETA.DIST pdf at a, b > 1 with x where both Loader "
            "bd0 sides take their log form; column A the density, B/C/D echo x, a, b (parser check).",
            "tags": ["M9", "probe", "statistics", "T-2513", "incomplete-beta"],
            "sheets": [{"name": "Calc", "cells": cells}],
            "expected": {},
        }
    return out


# ------------------------------------------------------------------ data
_RC = HERE / "rows_cache.pkl"


def load(which: str):
    """Rows (dicts blk, x, a, b, excel) of probe `which` with the parser echo checked."""
    cache = HERE / f"rows_{which}.pkl"
    if cache.exists():
        return pickle.loads(cache.read_bytes())
    rows, bad, fixed = [], 0, 0
    for part, chunk in enumerate(split(design(which)), 1):
        sc = CASES / f"{case_name(which, part)}.oracle.json"
        sh = json.load(open(sc))["sheets"]["Calc"]
        # w2's MP.num counts "20.29999999999998" as 15 digits (it strips any "0."), so a few shape literals
        # >= 10 were truncated by Excel's parser: take a and b from their echo cells (the same text as in A).
        real: dict = {}
        for i, (blk, x, a, b) in enumerate(chunk, 1):
            if f"C{i}" in sh:
                real[(a, b)] = (float(sh[f"C{i}"]["v"]), float(sh[f"D{i}"]["v"]))
        for i, (blk, x, a, b) in enumerate(chunk, 1):
            c = sh.get(f"A{i}")
            e = sh.get(f"B{i}")
            if e is not None and float(e["v"]) != x:
                bad += 1
            ra, rb = real[(a, b)]
            fixed += (ra, rb) != (a, b)
            if c and c.get("t") == "n":
                rows.append(dict(blk=blk, x=x, a=ra, b=rb, excel=float(c["v"])))
    print(f"{which}: {len(rows)} rows; x echo mismatches {bad}; rows with a parser-truncated shape {fixed}")
    cache.write_bytes(pickle.dumps(rows))
    return rows


def r2_rows():
    """r2-density-large's 32,264 a, b > 1 pdf cells (round-1 data plus its p1 probe), with exact values."""
    return L.load(include_probes=True)


# ------------------------------------------------------------------ X part and inversion
_XC_P = HERE / "xcache.pkl"
_XC: dict = pickle.loads(_XC_P.read_bytes()) if _XC_P.exists() else {}


def xpart(x, a, b):
    key = (x, a, b)
    if key not in _XC:
        _XC[key] = LD.xpart(x, a, b, **BEST_X)[0]
    return _XC[key]


def save_cache():
    _XC_P.write_bytes(pickle.dumps(_XC))


def eset(y: float, span: int = 8):
    """All doubles E with EXP_V6(E) == y (scan around fl(ln y))."""
    e0 = float(mpmath.log(mpf(y)))
    out = []
    for k in range(-span, span + 1):
        e = MP.step(e0, k) if k else e0
        if L.exp_v6(mpf(e)) == y:
            out.append(e)
    return out


def c_interval(y: float, X):
    """Constraint on a real C under E = fl53(C + X) (X extended): [lo, hi] or None."""
    es = eset(y)
    if not es:
        return None
    lo, hi = min(es), max(es)
    dlo = (lo - math.nextafter(lo, -math.inf)) / 2  # half-gaps (they differ at a power of two)
    dhi = (math.nextafter(hi, math.inf) - hi) / 2
    return (mpf(lo) - dlo - X, mpf(hi) + dhi - X)


def exact_const(a: float, b: float, form: str = "R"):
    """The exact Loader constant at the stored shapes, ln pdf - X: Cex = -ln B(k+1, nk+1) + k ln(k/n) + nk ln(nk/n),
    with k, n, nk the stored doubles (ln(a+b-1) is inside -ln B)."""
    k, n, nk = (mpf(v) for v in args_form(a, b, form))
    A, B = k + 1, nk + 1
    return (
        - (mpmath.loggamma(A) + mpmath.loggamma(B) - mpmath.loggamma(A + B))
        + k * mpmath.log(k / n)
        + nk * mpmath.log(nk / n)
    )


def units(v) -> float:
    """v in units of 2^-53 (absolute, the exponent's scale)."""
    return float(mpf(v) * mpf(2) ** 53)


if __name__ == "__main__":
    for w in ("sweep", "diverse", "hold", "grid"):
        cs = build_cases(w)
        for name, c in cs.items():
            p = CASES / f"{name}.case.json"
            print(name, len(c["sheets"][0]["cells"]), "exists" if p.exists() else "new")
            if "--write" in sys.argv and not p.exists():
                p.write_text(json.dumps(c, indent=1) + "\n")


def _t_at(t0, i):
    """The i-th 64-bit (extended) value from t0."""
    m, e = mpmath.frexp(t0)
    ul = mpf(2) ** (e - 64)
    return t0 + i * ul


def e_interval_ext(y: float):
    """Real interval of an extended E with EXP_V6(E) == y (t = RN64(E log2 e), the V6 tail on t; bisection on the
    64-bit grid of t, monotone). None if no t gives y."""
    t0 = L.R64(mpmath.log(mpf(y)) / mpmath.ln(2))
    m, e = mpmath.frexp(t0)
    ul = mpf(2) ** (e - 64)

    def out(i):
        return L.W1.exp2_v6(t0 + i * ul)

    def first(pred, lo, hi):  # smallest i in (lo, hi] with pred(i); pred(lo) false, pred(hi) true
        while hi - lo > 1:
            mid = (lo + hi) // 2
            if pred(mid):
                hi = mid
            else:
                lo = mid
        return hi

    # cover +-16 output ulps: t steps of ul against y's relative spacing 2^-52 (t = log2 y, dt = dy/y/ln 2)
    span = max(1 << 16, int(mpmath.ceil(mpf(2) ** -48 / ul)))
    if not (out(-span) < y < out(span)):
        return None
    ilo = first(lambda i: out(i) >= y, -span, span)
    ihi = first(lambda i: out(i) > y, -span, span) - 1
    if ihi < ilo or out(ilo) != y:
        return None
    lg = L.W1.LOG2E64
    return ((t0 + (ilo - mpf(0.5)) * ul) / lg, (t0 + (ihi + mpf(0.5)) * ul) / lg)


# ------------------------------------------------------------------ argument forms of k, n, n-k
def args_form(a: float, b: float, form: str):
    """(k, n, nk) as doubles under an argument form: 'R' k = a-1, n = fl(a+b)-2, nk = n-k (R's dbinom_raw);
    'B' k = a-1, nk = b-1, n = fl(a+b)-2;  'S' k = a-1, nk = b-1, n = fl(k+nk);  'Re' as R with n-k unrounded;
    'E' n = a+b-2 and nk = b-1 unrounded (extended)."""
    D = L.D
    k = D(mpf(a) - 1)
    if form == "E":  # n = a + b - 2 and n - k = b - 1 kept extended (exact here)
        return k, mpf(a) + mpf(b) - 2, mpf(b) - 1
    if form == "Re":  # n - k left unrounded (extended register; exact here)
        n = D(mpf(D(mpf(a) + mpf(b))) - 2)
        return k, n, mpf(n) - mpf(k)
    if form == "R":
        n = D(mpf(D(mpf(a) + mpf(b))) - 2)
        return k, n, D(mpf(n) - mpf(k))
    nk = D(mpf(b) - 1)
    if form == "B":
        return k, D(mpf(D(mpf(a) + mpf(b))) - 2), nk
    return k, D(mpf(k) + mpf(nk)), nk


_XV: dict = {}


def xpart_v(x: float, a: float, b: float, form: str = "R"):
    """r2's x87 X = -bd0(k, fl(n x)) - bd0(nk, fl(n fl(1-x))) (log form t = fl(k LN(fl(k/M))) + (M - k) extended,
    cut 0.2) with (k, n, nk) from args_form."""
    key = (x, a, b, form)
    if key not in _XV:
        k, n, nk = (mpf(v) for v in args_form(a, b, form))
        kw = {kk: v for kk, v in BEST_X.items() if kk not in ("ar", "lnmode", "xar")}
        X = mpf(x)
        q = mpf(1.0 - x)
        b1 = LD.bd0(k, L.rnd(n * X, "x"), "x", "xl", **kw)
        b2 = LD.bd0(nk, L.rnd(n * q, "x"), "x", "xl", **kw)
        _XV[key] = L.rnd(-b1 - b2, "e")
    return _XV[key]


# ------------------------------------------------------------------ Excel GAMMALN (own cache, this directory)
_GLP = HERE / "gl_cache.pkl"
_GLC: dict = pickle.loads(_GLP.read_bytes()) if _GLP.exists() else {}


def gl_prefetch(vs):
    need = sorted({float(v) for v in vs if float(v) not in _GLC})
    for i in range(0, len(need), 20000):
        ch = need[i : i + 20000]
        for v, r in zip(ch, L.excelmath.excel("GAMMALN", ch)):
            _GLC[v] = r
    if need:
        _GLP.write_bytes(pickle.dumps(_GLC))


def GL(v: float) -> float:
    if float(v) not in _GLC:
        gl_prefetch([v])
    return _GLC[float(v)]


# ------------------------------------------------------------------ log-form variants of t = k ln(k/M)
LN2_64 = L.W1.LN2_64


def t_log(k, M, tmode: str):
    """t = k ln(k/M) in bd0's log form; r = fl(k/M) stored.  'V0' fl(k * LN(r)) (r2's model); 'V1' fl(k * ln64(r))
    (LN left extended); 'V2' FYL2X with y = RN64(k ln2): fl(RN64(RN64(k ln2) log2 r)); 'V3' fl(RN64(k * ln64(r)))
    (product rounded to 64 bits first); 'V4' fl(RN64(k * LN(r))); 'V5' fl(k * RN53(ln r)) (correctly rounded log);
    'V6' RN53(k * RN53(ln r)); 'V7' fl(k fl(LN k - LN M)); 'V8' fl(fl(k LN k) - fl(k LN M))."""
    k, M = mpf(k), mpf(M)
    r = mpf(L.D(k / M))
    if tmode == "V0":
        return mpf(L.D(k * mpf(L.LN(float(r)))))
    if tmode == "V1":
        return mpf(L.D(k * L.ln64(r)))
    if tmode == "V2":
        y = L.R64(k * LN2_64)
        return mpf(L.D(L.W1.fyl2x(y, r)))
    if tmode == "V5":  # correctly rounded log (a CRT log, not Excel's worksheet LN)
        return mpf(L.D(k * mpf(L.R53(mpmath.log(r)))))
    if tmode == "V6":  # correctly rounded log and product in SSE double (single rounding)
        return mpf(L.R53(k * mpf(L.R53(mpmath.log(r)))))
    if tmode == "V7":  # ln(k/M) as LN(k) - LN(M)
        return mpf(L.D(k * mpf(L.D(mpf(L.LN(float(k))) - mpf(L.LN(float(M)))))))
    if tmode == "V8":  # k LN(k) - k LN(M)
        return mpf(L.D(mpf(L.D(k * mpf(L.LN(float(k))))) - mpf(L.D(k * mpf(L.LN(float(M)))))))
    if tmode == "V4":
        return mpf(L.D(L.R64(k * mpf(L.LN(float(r))))))
    return mpf(L.D(L.R64(k * L.ln64(r))))


_XT: dict = {}


def xpart_t(x: float, a: float, b: float, tmode: str, mmode: str = "nx", bd0st: bool = False):
    """X with both bd0 sides in log form (the probe's design) and t from t_log; bd0 = t + (M - k) extended.
    mmode: 'nx' M1 = fl(n x), M2 = fl(n fl(1-x)); 'ext' both unrounded; 'n-M1' M2 = fl(n - M1);
    'sx2' M = fl(fl((a+b) x) - 2x) and the same with fl(1-x); 'e1' M1 extended, M2 stored.
    bd0st: each bd0 value rounded to a double (fl53(fl64)) before the sum."""
    key = (x, a, b, tmode, mmode, bd0st)
    if key not in _XT:
        k, n, nk = (mpf(v) for v in shape_pieces(a, b))
        X, Q = mpf(x), mpf(1.0 - x)
        if mmode == "ext":
            M1, M2 = n * X, n * Q
        elif mmode == "e1":
            M1, M2 = n * X, mpf(L.D(n * Q))
        elif mmode == "n-M1":
            M1 = mpf(L.D(n * X))
            M2 = mpf(L.D(n - M1))
        elif mmode == "sx2":
            s = mpf(L.D(mpf(a) + mpf(b)))
            M1 = mpf(L.D(mpf(L.D(s * X)) - 2 * X))
            M2 = mpf(L.D(mpf(L.D(s * Q)) - 2 * Q))
        else:
            M1 = mpf(L.D(n * X))
            M2 = mpf(L.D(n * Q))
        b1 = t_log(k, M1, tmode) + (M1 - k)
        b2 = t_log(nk, M2, tmode) + (M2 - nk)
        if bd0st:  # bd0 returned as a double (MSVC /fp:precise rounds function returns)
            b1, b2 = mpf(L.D(b1)), mpf(L.D(b2))
        _XT[key] = L.R64(-b1 - b2)
    return _XT[key]
