"""T-2586 r4-f-small-df: shared helpers for the F density at small df.
Everything Excel-specific is imported: r4-f-brcomp `fblib` (loaders for r3's and r4-f-brcomp's sets, class scorer)
and `ratlib` (the log-form chain family, M-xd = the best round-4 model), r3-fdensity `flib` (exact F, Excel LN,
GAMMALN cache, tally), r3-loader-bd0 `r3lib` (x87 store, EXP V6 on an extended input), r3-tdensity `tdlib`
(Excel log1p). This directory's probes come from `mkprobe.rows`.
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
T = HERE.parent
for p in (T / "r4-f-brcomp", T / "r3-fdensity"):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))

import mpmath  # noqa: E402
import numpy as np  # noqa: E402

import fblib as B  # noqa: E402
import ratlib as RL  # noqa: E402

F = B.F
R3 = F.R3
TD = F.TD
LDB = np.longdouble
st = R3.st
mpf = mpmath.mpf
U = mpf(2) ** -53
_MP = B._load("r4sd_mkprobe", HERE / "mkprobe.py")

MXD = dict(pw="xd", hl="ext", grp="none", est="ext", L="xl")


def load(name: str):
    """Rows with 'excel', 'exact', 'cr'. Names: this directory's 'mya', 'myhold'; fblib's 'a', 'sw', 'cells',
    'big', 'hold4', 'hold', 'holdsw'; 'tdens' (r3-tdensity's column G, d1 = 1)."""
    if name in ("mya", "myhold"):
        rows = F.with_exact(_MP.rows(name[2:]))
    elif name == "tdens":
        rows = F.with_exact(F.tdens_rows())
    elif name.startswith("case:"):  # any probes-numeric case, parsed by verify-r3-fdensity's own vlib
        V = B._load("vlib_r4sd", T / "verify-r3-fdensity" / "vlib.py")
        rows = [dict(r, src=name, blk=r["case"]) for r in V.case_rows(name[5:])]
        rows = F.with_exact([r for r in rows if r["excel"] > 0 and r["x"] > 0 and r["d1"] >= 1 and r["d2"] >= 1])
    else:
        rows = B.load(name)
    return rows


def small(r) -> bool:
    return min(r["d1"], r["d2"]) <= 2


def D_units(model_val: float, r) -> float:
    """(model - exact)/exact in units of 2^-53."""
    return float((mpf(model_val) - r["exact"]) / r["exact"] / U)


def mxd(r) -> float:
    return RL.value(r["x"], r["d1"], r["d2"], MXD)


def side_r(x: float, m: float, n: float):
    """(side, r) as Excel forms them: mx = fl(m x); side 1 r = fl(mx/n), side 2 r = fl(n/mx)."""
    mx = st(LDB(m) * LDB(x))
    if mx <= n:
        return 1, st(LDB(mx) / LDB(n))
    return 2, st(LDB(n) / LDB(mx))


def L_xl(r: float) -> LDB:
    """Excel's log1p (series to 0.375, LN(fl(1+r)) above) as a longdouble."""
    return RL.L_of(r, "xl")


def EXP(E) -> float:
    return R3.exp_v6(E)


def LN(v) -> LDB:
    return F.LN(v)


def ld(v) -> LDB:
    return R3._ld_of(v)


def score_rows(rows, fn, by=None):
    """{key: tally}; fn(row) -> model; by(row) -> class key (an 'all' key is always added)."""
    out: dict = {}
    for r in rows:
        v = float(fn(r))
        for c in ("all",) + ((by(r),) if by else ()):
            out.setdefault(c, []).append((v, r["excel"]))
    return {c: F.tally(p) for c, p in out.items()}


def shape_key(r):
    return f"({r['d1']:g},{r['d2']:g})"


def w15(m: float, e: float) -> bool:
    return TD.w15(m, e)


def isfin(v) -> bool:
    return math.isfinite(float(v))


# ------------------------------------------------------------------ Excel GAMMALN / GAMMA / EXP / LN (harness), cached here
import pickle  # noqa: E402

_XF = HERE / "xl_cache.pkl"
_XL: dict = pickle.loads(_XF.read_bytes()) if _XF.exists() else {}


def xl(name: str, vs) -> list:
    """Excel's NAME(v) for doubles v (../excelmath, batched, cached in xl_cache.pkl)."""
    sys.path.insert(0, str(T))
    import excelmath

    need = sorted({float(v) for v in vs if (name, float(v)) not in _XL})
    if need:
        for v, r in zip(need, excelmath.excel(name, need)):
            _XL[(name, v)] = r
        _XF.write_bytes(pickle.dumps(_XL))
    return [_XL[(name, float(v))] for v in vs]


def GL(v: float) -> float:
    return xl("GAMMALN", [v])[0]


def GA(v: float) -> float:
    return xl("GAMMA", [v])[0]


def lnB_exact(m: float, n: float):
    return RL._lnB(m, n)


def K_exact(m: float, n: float, side: int):
    """Exact K_side = -lnB + a ln(m/n) (side 1) | -lnB - b ln(m/n) (side 2) (M-xd's constant)."""
    a, b = mpf(m) / 2, mpf(n) / 2
    t = mpmath.log(mpf(m) / mpf(n))
    return -lnB_exact(m, n) + (a * t if side == 1 else -b * t)


def mxd_K(x: float, m: float, n: float, K) -> float:
    """M-xd with the constant K (longdouble) supplied: F = fl(EXP(K + fl(c LN x) - h L(r)) / x)."""
    a, b, h = m / 2, n / 2, (m + n) / 2
    side, r = side_r(x, m, n)
    c = a if side == 1 else -b
    E = LDB(K) + LDB(st(LDB(c) * LN(x))) - LDB(h) * L_xl(r)
    return st(LDB(EXP(E)) / LDB(x))


# ------------------------------------------------------------------ vectorised EXP V6 (scans only; checked by s4 --control)
_LN2L = LDB(mpmath.nstr(mpmath.log(2), 30))


def exp_v6_vec(E: np.ndarray) -> np.ndarray:
    """EXP V6 on an array of extended E with numpy x87 ops; 2^|f| - 1 by expm1l (not always correctly rounded to
    64 bits, so a scan helper only: final scores use R3.exp_v6)."""
    t = E.astype(LDB) * R3.LOG2E64
    n = np.rint(t)
    f = t - n
    u = np.expm1(np.abs(f) * _LN2L)
    w = LDB(1) + u
    w = np.where(f < 0, LDB(1) / w, w)
    return np.ldexp(w, n.astype(int)).astype(np.float64)


def pieces_A(x: float, m: float, n: float) -> LDB:
    """M-xd's E minus K: fl(c LN x) - h L(r) (extended)."""
    a, b, h = m / 2, n / 2, (m + n) / 2
    side, r = side_r(x, m, n)
    c = a if side == 1 else -b
    return LDB(st(LDB(c) * LN(x))) - LDB(h) * L_xl(r)


# ------------------------------------------------------------------ generic log-form chains E = K + P - h L, F = EXP(E) [/ x]
def chain(x: float, m: float, n: float, pw: str, L_mode: str = "xl"):
    """(A, K_exact, div) for the power-term form pw (ratlib's names: x, xd, xn, r, rdiv, mx, mx1, exact):
    A = P - h L(r) as Excel would hold it (extended), K_exact the exact constant that makes the chain exact, div
    whether the value is divided by x at the end."""
    a, b, h = m / 2, n / 2, (m + n) / 2
    X, M, N = mpf(x), mpf(m), mpf(n)
    side, r = side_r(x, m, n)
    L = RL.L_of(r, L_mode)
    lnf = (M / 2) * mpmath.log(M * X / (M * X + N)) + (N / 2) * mpmath.log(N / (M * X + N)) - mpmath.log(X)
    lnf -= lnB_exact(m, n)
    Lr_ex = h * mpmath.log1p(M * X / N if side == 1 else N / (M * X))
    div = pw in ("xd", "rdiv")
    if pw == "exact":
        P_ex = ((a - 1) if side == 1 else -(b + 1)) * mpmath.log(X)
        P = ld(P_ex)
    elif pw == "x":
        c = (a - 1) if side == 1 else -(b + 1)
        P_ex, P = c * mpmath.log(X), LDB(st(LDB(c) * LN(x)))
    elif pw in ("xd", "xn"):
        c = a if side == 1 else -b
        P_ex, P = c * mpmath.log(X), LDB(st(LDB(c) * LN(x)))
        if pw == "xn":
            P_ex, P = P_ex - mpmath.log(X), P - LN(x)
    elif pw == "xn64":  # fl(c LN x) - ln x with the second ln x unstored (x87 FYL2X kept in the register)
        c = a if side == 1 else -b
        P_ex, P = c * mpmath.log(X) - mpmath.log(X), LDB(st(LDB(c) * LN(x))) - R3.LN64(x)
    elif pw == "xnex":  # fl(c LN x) - exact ln x (bound: the 1/x as an exact correction)
        c = a if side == 1 else -b
        P_ex, P = c * mpmath.log(X) - mpmath.log(X), LDB(st(LDB(c) * LN(x))) - ld(mpmath.log(X))
    elif pw in ("r", "rdiv"):
        c = a if side == 1 else b
        rex = M * X / N if side == 1 else N / (M * X)
        P_ex, P = c * mpmath.log(rex), LDB(st(LDB(c) * LN(r)))
        if pw == "r":
            P_ex, P = P_ex - mpmath.log(X), P - LN(x)
    elif pw == "mx":
        c = a if side == 1 else -b
        P_ex, P = c * mpmath.log(M * X) - mpmath.log(X), LDB(st(LDB(c) * LN(st(LDB(m) * LDB(x))))) - LN(x)
    elif pw == "mx1":
        c = (a - 1) if side == 1 else -(b + 1)
        P_ex, P = c * mpmath.log(M * X), LDB(st(LDB(c) * LN(st(LDB(m) * LDB(x)))))
    else:
        raise ValueError(pw)
    K_ex = lnf - P_ex + Lr_ex + (mpmath.log(X) if div else 0)
    return P - LDB(h) * L, K_ex, div
