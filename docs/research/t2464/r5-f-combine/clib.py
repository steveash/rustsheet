"""T-2586 r5-f-combine: shared helpers (everything Excel-specific is imported, nothing copied).

Imports r4-f-small-df `slib` (loaders for every F-density set, Excel LN / log1p / EXP V6, side and r as Excel forms
them) and `sdmodel` (M-sd's constant delta), r3-loader-bd0's `interval` (monotone bisection) through slib.
Model pieces of the stored-sum structure (r4-f-power):
  P  = +fl(a LN x) (side 1) | -fl(b LN x) (side 2), a double
  H  = -h L(r), extended (L = Excel's log1p: series to 0.375, LN(fl(1+r)) above)
  S1 = fl(K_hi + H), S2 = fl(S1 + P), E = S2 + K_lo, F = fl(EXP_V6(E)/x)
Excel's admissible E per cell is found by interval inversion of the final step (`e_interval`), in units 2^-53
relative to E0 = the extended nearest value of ln(exact F * x).
"""

from __future__ import annotations

import pickle
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
T = HERE.parent
for p in (T / "r4-f-small-df",):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))

import mpmath  # noqa: E402
import numpy as np  # noqa: E402

import sdmodel as SD  # noqa: E402
import slib as SL  # noqa: E402

LDB, st, mpf, R3, F = SL.LDB, SL.st, SL.mpf, SL.R3, SL.F
U = mpf(2) ** 53  # units per 1.0
_MK = SL.B._load("r5c_mkprobe", HERE / "mkprobe.py")


def ex(v):
    """Exact mpf of a double / longdouble / mpf."""
    if isinstance(v, mpmath.mpf):
        return v
    p, q = LDB(v).as_integer_ratio()
    return mpf(p) / q


def load(name: str):
    """'ca'/'chold' = this directory's probes; 'case:<id>' any probes-numeric case; else slib's names."""
    if name in ("ca", "chold"):
        return F.with_exact(_MK.rows(name[1:]))
    return SL.load(name)


def pieces(x: float, m: float, n: float) -> dict:
    a, b, h = m / 2, n / 2, (m + n) / 2
    side, r = SL.side_r(x, m, n)
    c, sg = (a, 1) if side == 1 else (b, -1)
    Lx = SL.LN(x)
    P = sg * st(LDB(c) * Lx)
    H = -LDB(h) * SL.L_xl(r)
    return dict(side=side, r=r, P=P, H=H, Lx=Lx, h=h, a=a, b=b)


# ------------------------------------------------------------------ Excel's admissible E (cached)
_IVF = HERE / "iv_cache.pkl"
_IV: dict = pickle.loads(_IVF.read_bytes()) if _IVF.exists() else {}
_dirty = [0]


FINS = ("div", "mulr", "lnx", "lnx64", "dbl", "exp")


def E0_of(r, fin: str = "div") -> LDB:
    """Reference E: ln(exact F * x) (the final steps that apply the 1/x themselves), ln(exact F) for 'exp'."""
    return SL.ld(mpmath.log(r["exact"] * (1 if fin == "exp" else mpf(r["x"]))))


def final(E, x: float, fin: str = "div") -> float:
    """Excel's last step from the extended exponent E (fin: 'div' fl(EXP(E)/x) with EXP's result extended; 'mulr'
    fl(EXP(E)*fl(1/x)); 'dbl' fl(fl(EXP(E))/x); 'lnx' EXP(E - LN(x)); 'lnx64' EXP(E - LN64(x)); 'exp' EXP(E)
    with E = ln F, the model holding the 1/x)."""
    X = LDB(x)
    if fin == "div":
        return st(R3.exp_v6(E, True) / X)
    if fin == "mulr":
        return st(R3.exp_v6(E, True) * LDB(st(LDB(1) / X)))
    if fin == "dbl":
        return st(LDB(R3.exp_v6(E)) / X)
    if fin == "lnx":
        return R3.exp_v6(E - SL.LN(x))
    if fin == "lnx64":
        return R3.exp_v6(E - R3.LN64(x))
    if fin == "exp":  # E already holds the -ln x
        return R3.exp_v6(E)
    raise ValueError(fin)


def e_interval(r, fin: str = "div", W: float = 2048.0):
    """(lo, hi) units offsets u with final(E0 + u) == excel, or None. E0 = E0_of(r, fin)."""
    key = (r["x"], r["d1"], r["d2"], r["excel"]) + ((fin,) if fin != "div" else ())
    if key in _IV:
        return _IV[key]
    E0, u1 = E0_of(r, fin), LDB(2.0) ** -53

    def fn(u):
        return final(E0 + LDB(u) * u1, r["x"], fin)

    iv = R3.interval(fn, r["excel"], W=W, tol=1e-3)
    _IV[key] = iv[0] if iv else None
    _dirty[0] += 1
    if _dirty[0] % 2000 == 0:
        save()
    return _IV[key]


def save():
    _IVF.write_bytes(pickle.dumps(_IV))


# ------------------------------------------------------------------ runs
def run_key(r) -> tuple:
    side, _ = SL.side_r(r["x"], r["d1"], r["d2"])
    return (r["d1"], r["d2"], side, int(round(float(np.log2(r["x"])) * 64)))


def runs(rows, branch: str = "log"):
    """{key: rows} per run (shape, side, x bucket); branch 'log' keeps r > 0.375, 'ser' r <= 0.375, 'all' both."""
    g: dict = {}
    for r in rows:
        _, rr = SL.side_r(r["x"], r["d1"], r["d2"])
        if branch == "log" and rr <= 0.375 or branch == "ser" and rr > 0.375:
            continue
        g.setdefault(run_key(r), []).append(r)
    return g


def K_exact(m: float, n: float, side: int):
    return SL.K_exact(m, n, side)


def K_sd(m: float, n: float, side: int):
    """M-sd's constant (exact K + delta units), mpf."""
    return K_exact(m, n, side) + mpf(SD.delta(m, n, side)) / U


# ------------------------------------------------------------------ the stored-sum model
def S2_of(Khi: float, pc: dict) -> float:
    S1 = st(LDB(Khi) + pc["H"])
    return st(LDB(S1) + LDB(pc["P"]))


def value(x: float, m: float, n: float, Khi: float, Ktot) -> float:
    """F = fl(EXP_V6(S2 + (Ktot - Khi)) / x), the remainder added to S2 in extended."""
    pc = pieces(x, m, n)
    E = ex(S2_of(Khi, pc)) + (mpf(Ktot) - ex(Khi))
    return final(SL.ld(E), x)


def best_overlap(ivs):
    """(count, lo, hi) of the deepest point of a list of closed intervals."""
    ev = sorted([(lo, 0) for lo, _ in ivs] + [(hi, 1) for _, hi in ivs])
    best, cur, at = 0, 0, None
    for i, (v, t) in enumerate(ev):
        cur += 1 if t == 0 else -1
        if t == 0 and cur > best:
            best, at = cur, i
    if at is None:
        return 0, None, None
    lo = ev[at][0]
    hi = next(v for v, t in ev[at + 1 :] if t == 1)
    return best, lo, hi


def run_cells(rs, fin: str = "div"):
    """Per cell: (pieces, E0 exact, (ulo, uhi)) for the cells with an admissible E under the final step fin."""
    out = []
    for r in rs:
        iv = e_interval(r, fin)
        if iv is None:
            continue
        out.append((pieces(r["x"], r["d1"], r["d2"]), ex(E0_of(r, fin)), iv, r))
    return out


def ktot_intervals(cells, Khi: float, Kref):
    """Per cell, the interval of delta_tot = (Ktot - Kref) units that reproduces Excel with this K_hi."""
    out = []
    kh = ex(Khi)
    for pc, E0, (lo, hi), _ in cells:
        k = float((E0 - ex(S2_of(Khi, pc)) + kh - Kref) * U)
        out.append((k + lo, k + hi))
    return out
