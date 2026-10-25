"""T-2513 r3-fdensity: shared helpers for Excel's F density F.DIST(x, d1, d2, FALSE).

Data sources (Excel values are always read from eval sidecars, never copied):
  - cells.tsv's F.DIST(.,FALSE) cells (pre-T2513 eval data), via `cells_rows()`;
  - r3-tdensity's probes, whose column G is F.DIST(t*t,1,n,FALSE) beside BETA.DIST(z,0.5,n/2,FALSE) (column H),
    via `tdens_rows()`;
  - this directory's probes (`mkprobe.py`), via `probe_rows()`.
Exact references are mpmath at 160 bits. Excel's arithmetic pieces come from r2-density-large `dlib`
(EXP V6, Excel LN, x87 rounding, scorer), w1-density (M1 pieces) and w2-forensics (exact encoder), by import.
"""

from __future__ import annotations

import functools
import json
import math
import pickle
import sys
from pathlib import Path

HERE = Path(__file__).parent
T = HERE.parent
for p in (
    T,
    T / "r2-density-large",
    T / "w2-forensics",
    T / "w1-density",
    T / "r2-log1p",
    T / "r3-loader-bd0",
    T / "r3-tdensity",
):
    if str(p) not in sys.path:
        sys.path.append(str(p))

import mpmath  # noqa: E402
import numpy as np  # noqa: E402

import r3lib as R3  # noqa: E402  (r3-loader-bd0: numpy x87 ops, Excel LN, EXP V6 on extended input, bd0, overlaps)
import tdlib as TD  # noqa: E402  (r3-tdensity: monotone interval inversion, stirlerr e-table reader)
import dlib as L  # noqa: E402  (r2-density-large: EXP V6, LN, x87 rounding, scorer)
import make_probes as MP  # noqa: E402  (w2-forensics encoder)

mpmath.mp.prec = 160
mpf = mpmath.mpf
D = L.D
R64 = L.R64
CASES = MP.CASES
TDENS = T / "r3-tdensity"


def sidecar(case: str) -> dict:
    p = CASES / f"{case}.oracle.json"
    return json.loads(p.read_text())["sheets"]["Calc"]


def val(sheet: dict, ref: str):
    c = sheet.get(ref)
    if c is None or c.get("t") != "n":
        return None
    return float(c["v"])


# ------------------------------------------------------------------ exact references
@functools.lru_cache(maxsize=None)
def _lnB(a, b):
    return mpmath.log(mpmath.beta(mpf(a), mpf(b)))


def f_exact(x: float, d1: float, d2: float):
    """Exact F density at the double x (mpf)."""
    X, m, n = mpf(x), mpf(d1), mpf(d2)
    lv = (m / 2) * mpmath.log(m * X / (m * X + n)) + (n / 2) * mpmath.log(n / (m * X + n)) - mpmath.log(X)
    return mpmath.exp(lv - _lnB(d1 / 2, d2 / 2))


def beta_exact(y: float, a: float, b: float):
    Y = mpf(y)
    return mpmath.exp((a - 1) * mpmath.log(Y) + (b - 1) * mpmath.log1p(-Y) - _lnB(a, b))


def cr(v) -> float:
    return float(v)


# ------------------------------------------------------------------ scoring
def tally(pairs):
    """(n, bit-exact, within 4 ulp, within 15 digits) of (model, excel) pairs."""
    return L.tally(pairs)


def fmt(t) -> str:
    return L.fmt(t)


def units(model: float, excel: float) -> float:
    """(model - excel)/excel in 2^-53 units (relative)."""
    if excel == 0 or not math.isfinite(model):
        return math.nan
    return float((mpf(model) - mpf(excel)) / mpf(excel) * mpf(2) ** 53)


# ------------------------------------------------------------------ data
def cells_rows():
    """cells.tsv F.DIST(x,d1,d2,FALSE) rows with numeric Excel values; d1, d2 truncated as Excel does."""
    out = []
    for line in (T / "cells.tsv").read_text().splitlines()[1:]:
        fn, case, cell, args, excel = line.split("\t")
        if fn != "F.DIST":
            continue
        a = args.split("|")
        if len(a) != 4 or a[3] != "False":
            continue
        try:
            x, d1, d2, ex = float(a[0]), float(a[1]), float(a[2]), float(excel)
        except ValueError:
            continue
        if not (x > 0 and d1 >= 1 and d2 >= 1 and math.isfinite(ex)) or ex == 0:
            continue
        out.append(
            dict(
                src="cells",
                blk=f"{case}:{cell}",
                x=x,
                d1=float(math.trunc(d1)),
                d2=float(math.trunc(d2)),
                excel=ex,
            )
        )
    return out


def tdens_rows(names=("a1", "a2")):
    """r3-tdensity probes: x = Excel's t*t (column C), F = column G, beta at z (column H)."""
    out = []
    for nm in names:
        meta = json.loads((TDENS / f"{nm}.meta.json").read_text())
        sh = sidecar(f"t2513-r3-tdensity-{nm}")
        for r in meta["rows"]:
            i, n = r["row"], r["n"]
            tt, G, H = val(sh, f"C{i}"), val(sh, f"G{i}"), val(sh, f"H{i}")
            if tt is None or G is None or not tt > 0:
                continue
            out.append(
                dict(src=f"tdens-{nm}", blk=f"{r['blk']}", x=tt, d1=1.0, d2=float(n), excel=G, beta_z=H)
            )
    return out


_PC = HERE / "cache.pkl"
_CACHE: dict = pickle.loads(_PC.read_bytes()) if _PC.exists() else {}


def cached(key, fn):
    if key not in _CACHE:
        _CACHE[key] = fn()
    return _CACHE[key]


def save_cache():
    _PC.write_bytes(pickle.dumps(_CACHE))


def with_exact(rows):
    for r in rows:
        r["exact"] = cached(("F", r["x"], r["d1"], r["d2"]), lambda r=r: f_exact(r["x"], r["d1"], r["d2"]))
        r["cr"] = float(r["exact"])
    save_cache()
    return rows


# ------------------------------------------------------------------ R's df (Loader dbinom_raw) with x87 pieces
LDB = np.longdouble
st = R3.st
U53 = LDB(2.0) ** -53


def _R(v, mode):
    return LDB(st(v)) if mode == "x" else LDB(v)


def stirlerr_exact(v):
    v = mpf(v)
    return mpmath.loggamma(v + 1) - (v + mpf(0.5)) * mpmath.log(v) + v - mpmath.log(mpmath.sqrt(2 * mpmath.pi))


@functools.lru_cache(maxsize=None)
def df_shape(m: float, n: float):
    """(k, N, Nk) of R df's dbinom_raw call, as Excel's stored doubles (exact for integer df)."""
    if m >= 2:
        return st((LDB(m) - 2) / 2), st((LDB(m) + LDB(n) - 2) / 2), None
    return st(LDB(m) / 2), st((LDB(m) + LDB(n)) / 2), None


@functools.lru_cache(maxsize=None)
def df_const_exact(m: float, n: float):
    """Loader's exact constant stirlerr(N) - stirlerr(k) - stirlerr(N-k) - lf/2 at R df's (k, N) (k > 0)."""
    k, N, _ = df_shape(m, n)
    k, N = mpf(k), mpf(N)
    lf = mpmath.log(2 * mpmath.pi) + mpmath.log(k) + mpmath.log(N - k) - mpmath.log(N)
    return stirlerr_exact(N) - stirlerr_exact(k) - stirlerr_exact(N - k) - lf / 2


def df_pq(x: float, m: float, n: float, o: dict):
    """R df's p, q and prefactor f, x87 ops stored by default.
    o['pq']: 'R' f0 = 1/(n + x m), q = n f0, p = x m f0 (R);  'div' q = n/(n + x m), p = x m/(n + x m);
             'w5' p = 1/(1 + n/(x m)), q = 1/(1 + x m/n) (w5's F cdf arguments);  'q1' q = fl(1 - p) after 'div'.
    o['pre']: storage of the prefactor ops ('x' stored, 'e' extended)."""
    X, M, N = LDB(x), LDB(m), LDB(n)
    xm = _R(X * M, "x")
    form = o.get("pq", "R")
    if form == "R":
        f0 = _R(LDB(1) / _R(N + xm, "x"), "x")
        q = _R(N * f0, "x")
        p = _R(xm * f0, "x")
    elif form in ("div", "q1", "p1"):
        den = _R(N + xm, "x")
        q = _R(N / den, "x")
        p = _R(xm / den, "x")
        if form == "q1":
            q = _R(LDB(1) - p, "x")
        if form == "p1":
            p = _R(LDB(1) - q, "x")
    elif form == "w5":
        p = _R(LDB(1) / _R(LDB(1) + _R(N / xm, "x"), "x"), "x")
        q = _R(LDB(1) / _R(LDB(1) + _R(xm / N, "x"), "x"), "x")
    else:
        raise ValueError(form)
    pa = o.get("pre", "x")
    if m >= 2:
        f = _R(_R(M * q, pa) / 2, pa)
    else:
        f = _R(_R(_R(M * M, pa) * q, pa) / _R(_R(_R(LDB(2) * p, pa) * _R(M + N, pa), pa), pa), pa)
    return p, q, f


def df_x(x: float, m: float, n: float, o: dict):
    """(-bd0(k, N p) - bd0(N-k, N q) as a longdouble, branches, p, q, f) for k > 0; bd0 knobs in o['bd0']."""
    p, q, f = df_pq(x, m, n, o)
    k, N, _ = df_shape(m, n)
    Nk = st(LDB(N) - LDB(k))
    ob = o.get("bd0", {})
    npx = _R(LDB(N) * p, o.get("np", "x"))
    nqx = _R(LDB(N) * q, o.get("np", "x"))
    b1, br1 = R3.bd0(k, npx, ob)
    b2, br2 = R3.bd0(Nk, nqx, ob)
    if ob.get("bst"):  # each bd0 returned as a double
        b1, b2 = LDB(st(b1)), LDB(st(b2))
    return -b1 - b2, br1 + br2, p, q, f


def df_out(X, f, C, o: dict) -> float:
    """f * EXP(C + X): E extended (o['E'] 'e') or stored ('x'); EXP's result stored ('x') or kept ('e')."""
    E = _R(LDB(C) + X, o.get("E", "e"))
    d = R3.exp_v6(E, True)
    d = _R(d, o.get("dens", "x"))
    if o.get("mul", "fd") == "fd":
        return st(f * d)
    return st(d * f)


def fit_df_block(rows, o: dict, W: float = 64.0, target=None):
    """Free constant for one (d1, d2) block under R df with knobs o: per-cell offset intervals (units 2^-53 from the
    exact constant) and the deepest overlap (count, offset).  `target` replaces Excel's values (positive controls)."""
    m, n = rows[0]["d1"], rows[0]["d2"]
    C0 = R3._ld_of(df_const_exact(m, n))
    ivs = []
    for j, r in enumerate(rows):
        X, _, _, _, f = df_x(r["x"], m, n, o)
        y = r["excel"] if target is None else target[j]
        ivs.append(TD.interval(lambda u, X=X, f=f: df_out(X, f, C0 + LDB(u) * U53, o), y, W))
    nb, pos = R3.best_overlap(ivs)
    return nb, pos, ivs, C0


def df_value(x: float, m: float, n: float, o: dict, off: float = 0.0) -> float:
    """Excel's F density under R df with knobs o and the exact Loader constant moved by `off` units.
    d1 = 2 takes dbinom_raw's x = 0 branch: lc = -bd0(N, N q) - N p for p < 0.1, else N LN(q) (o['k0'])."""
    k, N, _ = df_shape(m, n)
    if k == 0:
        p, q, f = df_pq(x, m, n, o)
        if p < 0.1 and o.get("k0", "R") == "R":
            b, _ = R3.bd0(N, _R(LDB(N) * q, "x"), o.get("bd0", {}))
            lc = -b - _R(LDB(N) * p, "x")
        else:
            lc = _R(LDB(N) * LDB(R3.LN(float(q))), "x")
        return df_out(lc, f, LDB(off) * U53, o)
    X, _, _, _, f = df_x(x, m, n, o)
    C = R3._ld_of(df_const_exact(m, n)) + LDB(off) * U53
    return df_out(X, f, C, o)


# ------------------------------------------------------------------ Excel GAMMALN (harness), cached here
def gl_prefetch(vs):
    import excelmath

    need = sorted({float(v) for v in vs if ("GL", float(v)) not in _CACHE and v > 0})
    for i in range(0, len(need), 20000):
        ch = need[i : i + 20000]
        for v, r in zip(ch, excelmath.excel("GAMMALN", ch)):
            _CACHE[("GL", v)] = r
    if need:
        save_cache()


def GL(v: float) -> float:
    v = float(v)
    if ("GL", v) not in _CACHE:
        gl_prefetch([v])
    return _CACHE[("GL", v)]


def LN(v) -> LDB:
    """Excel's LN of a double, as a longdouble holding the stored result."""
    return LDB(R3.LN(float(v)))


# ------------------------------------------------------------------ F as Excel's beta density at (y, q) times a Jacobian
def yq(x: float, m: float, n: float, form: str):
    """The transformed point and its separately formed complement, x87 stored ops.
    'w5': y = 1/(1 + n/(m x)), q = 1/(1 + m x/n) (w5's F cdf arguments); 'div': y = m x/(m x + n), q = n/(m x + n);
    'w5y': w5's y with q = fl(1 - y); 'divy': div's y with q = fl(1 - y)."""
    X, M, N = LDB(x), LDB(m), LDB(n)
    mx = _R(M * X, "x")
    if form.startswith("w5"):
        y = _R(LDB(1) / _R(LDB(1) + _R(N / mx, "x"), "x"), "x")
        q = _R(LDB(1) / _R(LDB(1) + _R(mx / N, "x"), "x"), "x")
    else:
        den = _R(N + mx, "x")
        y = _R(mx / den, "x")
        q = _R(N / den, "x")
    if form.endswith("y"):
        q = _R(LDB(1) - y, "x")
    return y, q


def loader_xq(y, q, a: float, b: float, ob: dict):
    """Loader's x part -bd0(k, n y) - bd0(nk, n q) with Excel's stored k, n, nk (r3-loader-bd0's bd0 knobs)."""
    k, n, nk, _ = R3.shape(a, b)
    b1, br1 = R3.bd0(k, _R(LDB(n) * y, "x"), ob)
    b2, br2 = R3.bd0(nk, _R(LDB(n) * q, "x"), ob)
    return -b1 - b2, br1 + br2


def jac_out(dens, x: float, m: float, n: float, y, q, jac: str) -> float:
    """Value-space Jacobian applied to the density (a longdouble holding Excel's stored density)."""
    X, M, N = LDB(x), LDB(m), LDB(n)
    if jac == "dyq/x":  # ((d y) q) / x
        return st(_R(_R(dens * y, "x") * q, "x") / X)
    if jac == "d(yq/x)":
        return st(dens * _R(_R(y * q, "x") / X, "x"))
    if jac == "(d yq)/x":
        return st(_R(dens * _R(y * q, "x"), "x") / X)
    if jac == "dq y/x":
        return st(_R(_R(dens * q, "x") * y, "x") / X)
    if jac == "mn/den2":  # dy/dx = m n/(m x + n)^2
        den = _R(N + _R(M * X, "x"), "x")
        return st(dens * _R(_R(_R(M * N, "x") / den, "x") / den, "x"))
    if jac == "mq2/n":  # m q^2 / n
        return st(dens * _R(_R(_R(M * q, "x") * q, "x") / N, "x"))
    raise ValueError(jac)
