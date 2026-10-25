"""T-2513 r3-loader-bd0: fast exact x87 arithmetic for Excel's Loader density (a, b > 1) and free-constant scans.

* x87 ops are numpy longdouble (64-bit significand, round to nearest) and `float()` stores (fl53(fl64(.))),
  which is exactly the playbook's stored x87 op.
* Excel LN is r2-density-large's model (w1 FLDLN2/FYL2X), memoised; Excel EXP is w1's V6 model.
* A cell is scored through its admissible exponent set S = {E double : EXP_V6(E) == excel} (cached), so a model
  only has to produce E.  A free per-block constant C is fitted by interval inversion: each cell's admissible C
  values form an interval, and the best C is the deepest overlap (exact, no grid).
* Data: r2-density-large's rows (dlib.load) plus this directory's probes (probe_<name>.meta.json + the case's
  oracle sidecar in eval/).
"""

from __future__ import annotations

import hashlib
import json
import math
import pickle
import sys
from fractions import Fraction
from pathlib import Path

import numpy as np

HERE = Path(__file__).parent
T = HERE.parent
sys.path.insert(0, str(T / "r2-density-large"))
import dlib as DL  # noqa: E402

import mpmath  # noqa: E402

CASES = Path("/home/user/rustsheet/eval/suites/probes-numeric/cases")
LDB = np.longdouble
TWO53 = Fraction(2**53)


# ---------------------------------------------------------------- arithmetic
def st(v) -> float:
    """Store an extended value to a double."""
    return float(v)


def R(v, mode: str):
    """'x' stored (double), 'e' extended (longdouble)."""
    return LDB(float(v)) if mode == "x" else v


_LN: dict = {}


def LN(v) -> float:
    """Excel's LN of a double."""
    v = float(v)
    r = _LN.get(v)
    if r is None:
        r = _LN[v] = DL.LN(v)
    return r


_LN64: dict = {}


def LN64(v):
    """FLDLN2; FYL2X kept extended (unstored), for an extended or double argument."""
    v = LDB(v)
    key = v.as_integer_ratio()
    r = _LN64.get(key)
    if r is None:
        q = DL.ln64(mpmath.mpf(key[0]) / key[1])
        r = _LN64[key] = _ld_of(q)
    return r


def frac(v) -> Fraction:
    return Fraction(*LDB(v).as_integer_ratio()) if not isinstance(v, Fraction) else v


def nextd(v: float, k: int = 1) -> float:
    for _ in range(abs(k)):
        v = math.nextafter(v, math.inf if k > 0 else -math.inf)
    return v


# ---------------------------------------------------------------- Excel EXP (model V6) on an extended input
LOG2E64 = LDB(int(DL.W1.LOG2E64.man)) * LDB(2.0) ** int(DL.W1.LOG2E64.exp)
_F2: dict = {}


def _ld_of(m) -> np.longdouble:
    m = mpmath.mpf(m)
    if m == 0:
        return LDB(0)
    sgn, man, ex, _ = m._mpf_
    return (LDB(-1) if sgn else LDB(1)) * LDB(int(man)) * LDB(2.0) ** int(ex)


def exp_v6(E, ext: bool = False):
    """w1's EXP V6 (t = RN64(E*RN64(log2 e)), n = nint(t), F2XM1 on |f|, reciprocal for f < 0) for a double or an
    extended E, with numpy x87 ops and one correctly rounded 2^|f| - 1 (mpmath)."""
    t = LDB(E) * LOG2E64
    n = np.rint(t)
    f = t - n
    af = abs(f)
    key = af.as_integer_ratio()
    u = _F2.get(key)
    if u is None:
        fm = mpmath.mpf(key[0]) / key[1]
        u = _F2[key] = _ld_of(DL.R64(mpmath.mpf(2) ** fm - 1))
    w = LDB(1) + u
    if f < 0:
        w = LDB(1) / w
    r = np.ldexp(w, int(n))
    return r if ext else float(r)


# ---------------------------------------------------------------- admissible E sets
_ES_PATH = HERE / "eset_cache.pkl"
_ES: dict = pickle.loads(_ES_PATH.read_bytes()) if _ES_PATH.exists() else {}
_ES_dirty = [0]


def eset(y: float) -> tuple:
    """Sorted doubles E with EXP_V6(E) == y (window scan around ln y; widened while the edge still matches)."""
    r = _ES.get(y)
    if r is not None:
        return r
    e0 = float(mpmath.log(y))
    ok = []
    W = 8
    while True:
        cand = [nextd(e0, j) for j in range(-W, W + 1)]
        ok = [e for e in cand if DL.exp_v6(mpmath.mpf(e)) == y]
        if not ok or (ok[0] != cand[0] and ok[-1] != cand[-1]) or W > 4096:
            break
        W *= 4
    r = _ES[y] = tuple(ok)
    _ES_dirty[0] += 1
    return r


def save_cache():
    if _ES_dirty[0]:
        _ES_PATH.write_bytes(pickle.dumps(_ES))
        _ES_dirty[0] = 0


def groups(es):
    """Contiguous runs [lo, hi] of an E set."""
    out = []
    for e in es:
        if out and nextd(out[-1][1]) == e:
            out[-1][1] = e
        else:
            out.append([e, e])
    return out


# ---------------------------------------------------------------- data
def holdout(blk, x, a, b) -> bool:
    return DL.holdout(blk, x, a, b)


def load_probe(name: str):
    """Rows of one of this directory's probes: meta probe_<name>.meta.json, values from the eval oracle sidecar."""
    meta = json.loads((HERE / f"probe_{name}.meta.json").read_text())
    side = CASES / f"{meta['case']}.oracle.json"
    if not side.exists():
        return []
    sh = json.loads(side.read_text())["sheets"]["Calc"]
    rows = []
    cells = []
    if "blocks" in meta:  # mkprobe.py: blocks of consecutive doubles, cells A1.. in order
        j = 0
        for blk, a, b, x0, cnt in meta["blocks"]:
            x = x0
            for _ in range(cnt):
                j += 1
                cells.append((f"A{j}", blk, x, a, b))
                x = math.nextafter(x, math.inf)
    else:
        cells = meta["rows"]
    for cell, blk, x, a, b in cells:
        c = sh.get(cell)
        if c and c.get("t") == "n":
            rows.append(dict(src=name, blk=blk, x=x, a=a, b=b, excel=float(c["v"]), cell=cell))
    for r in rows:
        r["hold"] = holdout(r["blk"], r["x"], r["a"], r["b"])
    return rows


def load_old():
    return DL.load()


def blocks(rows):
    out: dict = {}
    for i, r in enumerate(rows):
        out.setdefault(r["blk"], []).append(i)
    return out


# ---------------------------------------------------------------- Loader bd0 / x part
CS = [None] + [1.0 / (2 * j + 1) for j in range(1, 200)]  # RN(1/(2j+1))


def series(k, M, d, sm, o):
    if o.get("swap"):  # bd0(M, k): Loader's series with the arguments exchanged
        o2 = dict(o)
        o2.pop("swap")
        return series(M, k, -d, sm, o2)
    """bd0's small-ratio branch.  k, M doubles; d = k - M and sm = k + M (as computed).  Knobs in o:
    v, s, ej, t, v2: 'x' stored or 'e' extended for v = d/sm, the partial sums, ej, each term, v*v;
    coef: 'div' term = ej/(2j+1) (R) or 'mul' term = ej*RN(1/(2j+1));  nt: fixed term count (0 = R's test);
    s0: 'dv' s = d*v (R) or '2kv' s = (2k)*v*v ... ; ej0: '2x' ej = (2k)*v (R) or 'xs' ej = k*... (see code)."""
    ser = o.get("ser", "R")
    if ser in ("lp1", "lp2", "lp3"):  # k*log1p(d/M) with Excel's log1p (r2-log1p), then + (M - k)
        sys.path.insert(0, str(T / "r2-log1p"))
        import excel_log1p as XL

        a_ = st(d / M) if ser != "lp3" else st(LDB(st(k / M)) - 1)
        if (-0.375 <= a_ < 0) or ser == "lp2":
            l_ = XL.log1p_series(a_)
        else:
            l_ = LN(st(1 + LDB(a_)))
        t = LDB(st(k * LDB(l_)))
        return t - d
    if ser in ("atv", "atv2"):  # k*ln(k/M) by Excel's atanh series in v = d/sm (r2-log1p form), then - d
        v = LDB(st(d / sm))
        t2 = LDB(st(v * v))
        p_, s_ = v, None
        for j in range(1, o.get("nt", 14) + 1):
            p_ = LDB(st(p_ * t2))
            tm = LDB(st(p_ * LDB(CS[j])))
            s_ = tm if s_ is None else LDB(st(s_ + tm))
        l_ = LDB(st(LDB(2) * LDB(st(v + s_))))
        t = LDB(st(k * l_))
        return t - d if ser == "atv" else LDB(st(t - d))
    v = R(d / sm, o.get("v", "e"))
    ej = R(R(LDB(2) * (M if o.get("ejx") == "M" else k), "e") * v, o.get("ej", "e"))
    s0 = o.get("s0", "dv")
    if s0 == "dv":  # R: s = d*v
        s = R(d * v, o.get("s", "e"))
    elif s0 == "dd":  # Loader 2000: s = (x-np)*(x-np)/(x+np)
        s = R(R(d * d, o.get("dd", "x")) / sm, o.get("s", "e"))
    elif s0 == "ejd":  # s = ej - d (2kv - d, cancellation)
        s = R(R(ej, o.get("ej0", "x")) - d, o.get("s", "e"))
    else:  # 'ejd2': s = d - ej... (sign control)
        s = R(d - R(ej, o.get("ej0", "x")), o.get("s", "e"))
    v2 = R(v * v, o.get("v2", "e"))
    ph = LDB(0)
    if o.get("ph"):  # phenomenological: bd0 carries minus the rounding error of fl(2k*v)
        e2 = LDB(2) * k * v
        ph = LDB(st(e2)) - e2
    s = s - ph
    nt = o.get("nt", 0)
    coef = o.get("coef", "div")
    sm_ = o.get("s", "e")
    for j in range(1, 1000):
        ej = R(ej * v2, o.get("ej", "e"))
        t = R(ej / LDB(2 * j + 1) if coef == "div" else ej * LDB(CS[j]), o.get("t", "e"))
        s1 = R(s + t, sm_)
        if nt:
            s = s1
            if j >= nt:
                return s
            continue
        if (float(s1) == float(s)) if o.get("cmp", "own") == "x" else (s1 == s):
            return s1
        s = s1
    return s


def bd0(k, M, o: dict):
    """Excel's bd0(k, M): series when |d| < cut*(k+M), else t = fl(k*LN(fl(k/M))) + (M - k) extended."""
    k_ = LDB(k)
    M_ = LDB(M)
    d = R(k_ - M_, o.get("d", "x"))
    sm = R(k_ + M_, o.get("sm", "x"))
    if abs(d) < LDB(o.get("cut", 0.2)) * sm:
        return series(k_, M_, d, sm, o), "s"
    q = st(k_ / M_) if o.get("rq", "x") == "x" else k_ / M_
    lnq = LDB(LN(q)) if o.get("rq", "x") == "x" and o.get("lnm", "x") == "x" else LN64(q)
    t = R(k_ * lnq, o.get("tm", "x"))
    ld = o.get("ld", "e")
    if ld == "x":  # M - k stored (it rounds once the ratio is beyond Sterbenz's [1/2, 2])
        return t + LDB(st(M_ - k_)), "l"
    if ld == "R":  # R: fl(fl(t + M) - k)
        return LDB(st(LDB(st(t + M_)) - k_)), "l"
    if ld == "xx":  # fl(t + fl(M - k))
        return LDB(st(t + LDB(st(M_ - k_)))), "l"
    return t + (M_ - k_), "l"


def shape(a: float, b: float):
    """k = a-1, n = fl(a+b)-2, nk = n-k (stored)."""
    k = st(LDB(a) - 1)
    ab = st(LDB(a) + LDB(b))
    n = st(LDB(ab) - 2)
    nk = st(LDB(n) - LDB(k))
    return k, n, nk, ab


def xpart(x: float, a: float, b: float, o: dict):
    """(bd0(k, np), bd0(nk, nq), branches).  o['q']: 'fl' q = fl(1-x), 'ex' exact 1-x (extended);
    o['nq']: 'mul' nq = fl(n*q), 'sub' nq = fl(n - np); o['np']: 'x' stored n*x or 'e' extended."""
    k, n, nk, ab = shape(a, b)
    if o.get("n", "x") == "e":  # n = a+b-2 and nk = b-1 kept extended (exact)
        n = LDB(a) + LDB(b) - 2
        nk = LDB(b) - 1
    elif o.get("n") == "b":  # nk = b-1 directly, n stored
        nk = st(LDB(b) - 1)
    X = LDB(x)
    q = LDB(st(1 - X)) if o.get("q", "fl") == "fl" else LDB(1) - X
    npx = R(LDB(n) * X, o.get("np", "x"))
    if o.get("nq", "mul") == "mul":
        nqx = R(LDB(n) * q, o.get("np", "x"))
    else:
        nqx = R(LDB(n) - npx, o.get("np", "x"))
    b1, br1 = bd0(k, npx, o)
    b2, br2 = bd0(nk, nqx, o)
    return b1, b2, br1 + br2


# ---------------------------------------------------------------- constants of the chain
def shape_pieces(a, b):
    """L5 = LN(fl(a+b)-1), h = lf/2 with lf = LF0 = fl(fl(fl(ln2pi + LN k) + LN nk) - LN n) (x87 stored)."""
    k, n, nk, ab = shape(a, b)
    L5 = LN(st(LDB(ab) - 1))
    ln2pi = float(mpmath.log(2 * mpmath.pi))
    lf = st(LDB(st(LDB(st(LDB(ln2pi) + LDB(LN(k)))) + LDB(LN(nk)))) - LDB(LN(n)))
    return L5, lf / 2


def exact_logpdf(x, a, b):
    X, A, B = mpmath.mpf(x), mpmath.mpf(a), mpmath.mpf(b)
    return (A - 1) * mpmath.log(X) + (B - 1) * mpmath.log1p(-X) - mpmath.log(mpmath.beta(A, B))


# ---------------------------------------------------------------- free-constant fits by monotone inversion
U53 = LDB(2.0) ** -53


def chain_E(name: str, C, b1, b2, L5, h):
    """Exponent E from constant C (extended) and the two bd0 values, per chain.  A trailing 'e' keeps the last
    step extended (EXP of a 64-bit E); otherwise E is stored."""
    fin = (lambda v: v) if name.endswith("e") else (lambda v: LDB(st(v)))
    nm = name[:-1] if name.endswith("e") else name
    X = -b1 - b2
    if nm == "C":  # E = C + X
        return fin(C + X)
    if nm == "R":  # lc = fl(C + X); E = L5 + fl(lc - h)
        lc = LDB(st(C + X))
        return fin(LDB(L5) + LDB(st(lc - LDB(h))))
    if nm == "Rg":  # lc stored, g = lc - h extended, E = L5 + g
        lc = LDB(st(C + X))
        return fin(LDB(L5) + (lc - LDB(h)))
    if nm == "Rl":  # lc extended, g stored
        return fin(LDB(L5) + LDB(st(C + X - LDB(h))))
    if nm == "K":  # lc stored; E = fl(L5 - h) + lc
        lc = LDB(st(C + X))
        return fin(LDB(st(LDB(L5) - LDB(h))) + lc)
    if nm == "S":  # lc = fl(fl(C - b1) - b2); E = L5 + fl(lc - h)
        lc = LDB(st(LDB(st(C - b1)) - b2))
        return fin(LDB(L5) + LDB(st(lc - LDB(h))))
    if nm == "SC":  # E = fl(C - b1) - b2
        return fin(LDB(st(C - b1)) - b2)
    raise ValueError(name)


def chain_out(name: str, C, b1, b2, L5, h, ab1):
    """Output value of the density for a chain: E chains go through exp_v6; 'V*' chains multiply in value space."""
    if not name.startswith("V"):
        return exp_v6(chain_E(name, C, b1, b2, L5, h))
    X = -b1 - b2
    if name == "V0":  # out = fl(EXP(fl(C + X)) * (a+b-1))
        return st(LDB(exp_v6(st(C + X))) * LDB(ab1))
    if name == "V2":  # lc = fl(C + X); g = fl(lc - h); out = fl(EXP(g) * (a+b-1))
        lc = LDB(st(C + X))
        return st(LDB(exp_v6(st(lc - LDB(h)))) * LDB(ab1))
    if name == "V2e":  # same, EXP's result kept extended
        lc = LDB(st(C + X))
        return st(exp_v6(st(lc - LDB(h)), True) * LDB(ab1))
    if name == "V0e":
        return st(exp_v6(st(C + X), True) * LDB(ab1))
    if name == "V0x":  # EXP of an extended argument, result extended
        return st(exp_v6(C + X, True) * LDB(ab1))
    raise ValueError(name)


def off_interval(F, elo: float, ehi: float, W: float = 96.0, tol: float = 1e-5):
    """Offsets u in [-W, W] (units 2^-53) with elo <= F(u) <= ehi, F non-decreasing; None if empty."""
    a, b = -W, W
    if F(b) < elo or F(a) > ehi:
        return None
    lo, hi = a, b  # smallest u with F(u) >= elo
    while hi - lo > tol:
        m = (lo + hi) / 2
        if F(m) >= elo:
            hi = m
        else:
            lo = m
    umin = hi
    lo, hi = a, b  # largest u with F(u) <= ehi
    while hi - lo > tol:
        m = (lo + hi) / 2
        if F(m) <= ehi:
            lo = m
        else:
            hi = m
    umax = lo
    return (umin, umax) if umin <= umax else None


def interval(F, y: float, W: float = 96.0, tol: float = 1e-5):
    """Offsets u in [-W, W] (units 2^-53) with F(u) == y for a non-decreasing F, as [(lo, hi)] or [] (bisection to
    tol; also used by r3-tdensity's tdlib)."""
    a, b = -W, W
    if F(b) < y or F(a) > y:
        return []
    lo, hi = a, b
    while hi - lo > tol:
        m = (lo + hi) / 2
        if F(m) >= y:
            hi = m
        else:
            lo = m
    umin = hi
    lo, hi = umin, b
    while hi - lo > tol:
        m = (lo + hi) / 2
        if F(m) <= y:
            lo = m
        else:
            hi = m
    return [(umin, lo)] if umin <= lo and F(umin) == y else []


def cell_intervals(row, b1, b2, chain: str, C0, L5, h, W=96.0, tol=1e-5):
    """Offsets u (units 2^-53, C = C0 + u 2^-53) with EXP_V6(E(C)) == excel, as one interval (monotone map)."""
    C0_ = LDB(C0)
    y = row["excel"]

    ab1 = st(LDB(row["a"]) + LDB(row["b"]) - 1) if chain.startswith("V") else None
    ab1 = st(LDB(st(LDB(row["a"]) + LDB(row["b"]))) - 1) if ab1 is not None else None

    def F(u):
        return chain_out(chain, C0_ + LDB(u) * U53, b1, b2, L5, h, ab1)

    return interval(F, y, W, tol)


def best_overlap(ivs):
    """ivs: list of interval lists (one per cell). Returns (count, centre) of the deepest overlap."""
    ev = []
    for cell in ivs:
        for lo, hi in cell:
            ev.append((lo, 0))
            ev.append((hi, 1))
    ev.sort()
    c = best = 0
    pos = 0.0
    for i, (p, t) in enumerate(ev):
        if t == 0:
            c += 1
            if c > best:
                best = c
                nxt = ev[i + 1][0] if i + 1 < len(ev) else p
                pos = (p + nxt) / 2
        else:
            c -= 1
    return best, pos


def hits_at(ivs, c: float) -> int:
    return sum(any(lo <= c <= hi for lo, hi in cell) for cell in ivs)


def fit_block(rows, idx, o: dict, chain: str = "C", W: float = 1024.0):
    """Fit C on the even half, score the hold-out half (odd md5).  C0 is the exact-constant guess."""
    r0 = rows[idx[0]]
    a, b = r0["a"], r0["b"]
    L5, h = shape_pieces(a, b)
    Xs = {i: xpart(rows[i]["x"], a, b, o) for i in idx}
    i0 = idx[len(idx) // 2]
    lp = float(exact_logpdf(rows[i0]["x"], a, b))
    b1, b2, _ = Xs[i0]
    base = LDB(lp) + b1 + b2
    if chain.rstrip("e") in ("C", "SC"):
        C0 = base
    elif chain in ("V0", "V0e", "V0x"):
        C0 = base - LDB(L5)
    elif chain.startswith("V"):
        C0 = base - LDB(L5) + LDB(h)
    else:
        C0 = base - LDB(L5) + LDB(h)
    ivs = {i: cell_intervals(rows[i], Xs[i][0], Xs[i][1], chain, C0, L5, h, W) for i in idx}
    ev = [i for i in idx if not rows[i]["hold"]]
    od = [i for i in idx if rows[i]["hold"]]
    ne, ce = best_overlap([ivs[i] for i in ev])
    nh = hits_at([ivs[i] for i in od], ce)
    na, ca = best_overlap([ivs[i] for i in idx])
    br: dict = {}
    for i in idx:
        br[Xs[i][2]] = br.get(Xs[i][2], 0) + 1
    return dict(even=ne, nev=len(ev), hold=nh, nod=len(od), all=na, n=len(idx), off=ce, off_all=ca,
                br=br, C0=C0, ivs=ivs, X=Xs)


def md5(s: str) -> int:
    return hashlib.md5(s.encode()).digest()[0]
