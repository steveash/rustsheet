"""T-2586 r5-f-smallr: shared helpers.  Everything Excel-specific is imported (r4-f-power plib/cands, r3 flib,
r2-log1p excel_log1p, r4-f-small-df sdmodel); nothing is copied."""
from __future__ import annotations

import math
import sys
from pathlib import Path

T = Path(__file__).resolve().parent.parent  # docs/research/t2464
sys.path[:0] = [str(d) for d in (T / "r4-f-power", T / "r3-fdensity") if str(d) not in sys.path]

import mpmath  # noqa: E402

import cands as C  # noqa: E402  (r4-f-power: Excel stored pieces)
import plib as PL  # noqa: E402  (r4-f-power: loaders, runs)
import s3_order  # noqa: E402  (r4-f-power: exact K')

F = PL.F
LDB, st = F.LDB, F.st
mpf = mpmath.mpf
U = mpf(2) ** 53


def ulpu(v: float) -> float:
    """ulp of the double v in 2^-53 units (absolute)."""
    return math.ulp(abs(float(v))) * 2.0 ** 53 if v != 0 else 0.0


def base(x: float, m: float, n: float):
    """(side1, a, b, h, r, P, tau) with P = Excel's signed stored power product (as an mpf)."""
    a, b, h = m / 2, n / 2, (m + n) / 2
    mx = st(LDB(m) * LDB(x))
    side1 = mx <= n
    r = st(LDB(mx) / LDB(n)) if side1 else st(LDB(n) / LDB(mx))
    c, sg = (a, 1) if side1 else (b, -1)
    P = sg * mpf(st(LDB(c) * F.LN(x)))
    return side1, a, b, h, r, P


import numpy as np  # noqa: E402

LD = np.longdouble


def run_arrays(rs, m, n):
    """Per-run arrays: x, side1, r, P (double, signed), Lx = Excel LN(x), L = Excel log1p(r) (double), Y = ln(Excel x) - P
    in units relative to Y0 (float), Y0 (mpf), and the exact Y (same reference)."""
    xs, rr, Ps, Ls, Lx, Y, Yex = [], [], [], [], [], [], []
    s1 = None
    for r in rs:
        x = r["x"]
        side1, a, b, h, rat, P = base(x, m, n)
        s1 = side1
        X = mpf(x)
        rex = m * X / n if side1 else n / (m * X)
        Kp = s3_order.kprime(m, n, side1)
        tau = mpmath.log(mpf(r["excel"])) + mpmath.log(X)
        xs.append(x)
        rr.append(rat)
        Ps.append(float(P))
        Lx.append(float(F.LN(x)))
        Ls.append(float(F.TD.log1p_xl(rat)))
        Y.append(tau - P)
        Yex.append(Kp - h * mpmath.log1p(rex))
    Y0 = Yex[0]
    return dict(x=np.array(xs), side1=s1, r=np.array(rr), P=np.array(Ps), Lx=np.array(Lx), L=np.array(Ls),
                Y=np.array([float((y - Y0) * U) for y in Y]), Yex=np.array([float((y - Y0) * U) for y in Yex]),
                Y0=Y0, a=m / 2, b=n / 2, h=(m + n) / 2, Kp=s3_order.kprime(m, n, s1))


def f64(v):
    return np.asarray(v, dtype=LD).astype(np.float64)


def units_rel(v, Y0):
    """(v - Y0) in units, v an array of longdoubles/doubles; Y0 mpf.  Exact enough: split Y0 into hi + lo."""
    y0h = LD(float(Y0))
    y0l = float((Y0 - mpf(float(Y0))) * U)
    return (np.asarray(v, dtype=LD) - y0h).astype(np.float64) * 2.0 ** 53 - y0l


def load(src: str):
    """Rows of SRC: 'pw:<name>' r4-f-power's probes, 'r3:<name>' r3-fdensity's, 'br:<big|hold4>' r4-f-brcomp's,
    'own:<name>' this directory's probes."""
    kind, name = src.split(":", 1)
    if kind == "pw":
        return PL.own_rows(name)
    if kind == "r3":
        return PL.r3_rows(name)
    if kind == "br":
        if str(T / "r4-f-brcomp") not in sys.path:
            sys.path.append(str(T / "r4-f-brcomp"))
        import fblib

        return fblib.load(name)
    if kind == "own":
        import mkprobe5

        return F.with_exact(mkprobe5.rows(name))
    raise ValueError(src)


def runs(src: str, pred=None):
    """(blk, rows, d1, d2) per run of SRC; pred(m, n, rows) filters."""
    g: dict = {}
    for r in load(src):
        g.setdefault(r["blk"], []).append(r)
    for blk, rs in g.items():
        m, n = rs[0]["d1"], rs[0]["d2"]
        if pred is None or pred(m, n, rs):
            yield blk, rs, m, n


def small_r(m, n, rs, gmin=64.0):
    """Runs whose r stays <= 0.375 and whose S1 grid is >= gmin units (informative)."""
    x = rs[0]["x"]
    side1, a, b, h, r, P = base(x, m, n)
    if max(base(q["x"], m, n)[4] for q in rs) > 0.375:
        return False
    X = mpf(x)
    rex = m * X / n if side1 else n / (m * X)
    S1 = s3_order.kprime(m, n, side1) - h * mpmath.log1p(rex)
    return ulpu(float(S1)) >= gmin or ulpu(float(S1 + P)) >= gmin


# ====================================================================== chains (formerly chainlib.py)
# General stored-sum chains with a free constant, scored per run.  A chain is a string like 'C+H!+P!' : terms summed
# left to right, '!' after a term = the partial sum is stored to double there (x87 extended otherwise).  'C' is the free
# constant (scanned).  Terms per cell (all as they enter ln F, longdouble):
#   H  -h L(r) extended        Hs -fl(h L)        Ha -a L ext  Hb -b L ext  Has/Hbs stored products
#   P  the side's stored power (+fl(a LN x) side 1, -fl(b LN x) side 2)
#   Pa +fl(a LN x)  Pb -fl(b LN x)  Ph -fl(h LN x)  Pa1 +fl((a-1) LN x)  Pb1 -fl((b+1) LN x)  Ph1 -fl((h-1)LN x)
#   X  -LN(x) (then the value division by x is dropped)
#   Lr +fl(c LN r) (side 1 c = a, side 2 c = b)  Lmx: -fl(h LN(mx)) etc. are built on demand by name.
# Measured: ln(Excel) + ln(x) exactly (or ln(Excel) when X is in the chain).  Score: cells within TOL units of the
# median offset, maximised over C (grid STEP units over +-SPAN around the exact K').
import re  # noqa: E402

R = sys.modules[__name__]  # the chain code below addresses this module's helpers as R.*





TOL = 3.0


def parse(chain: str):
    toks = re.findall(r"([+-]?)([A-Za-z0-9_]+)(!?)", chain)
    return [(sg == "-", nm, bool(s)) for sg, nm, s in toks]


def cell_terms(x, m, n, names):
    a, b, h = m / 2, n / 2, (m + n) / 2
    side1, _, _, _, r, P = R.base(x, m, n)
    Lx = LD(R.F.LN(x))
    L = LD(R.F.TD.log1p_xl(r))

    def sp(c, v):
        return LD(st(LD(c) * v))

    t = {}
    for nm in names:
        if nm in ("C", "D"):
            continue
        if nm == "H":
            v = -LD(h) * L
        elif nm == "Hs":
            v = -sp(h, L)
        elif nm == "Ha":
            v = -LD(a) * L
        elif nm == "Hb":
            v = -LD(b) * L
        elif nm == "Has":
            v = -sp(a, L)
        elif nm == "Hbs":
            v = -sp(b, L)
        elif nm == "HxE":  # -h log1p(r) with the exact log1p of the stored r
            v = -LD(h) * R.F.R3._ld_of(mpmath.log1p(mpmath.mpf(r)))
        elif nm == "HxR":  # -h log1p(exact ratio)
            X = mpmath.mpf(x)
            v = -LD(h) * R.F.R3._ld_of(mpmath.log1p(m * X / n if side1 else n / (m * X)))
        elif nm.startswith("Hr_"):  # -h * Excel log1p of r formed another way (s3_forms.r_of forms + 'y')
            import s3_forms

            rf = nm[3:]
            if rf == "y":  # r from the beta variable: y = fl(mx / fl(n + mx)) (side 1 small side), r = fl(y / fl(1 - y))
                mx = st(LD(m) * LD(x))
                den = st(LD(n) + LD(mx))
                yy = st(LD(mx if side1 else n) / LD(den))
                rr2 = st(LD(yy) / LD(st(LD(1) - LD(yy))))
            elif rf == "d":  # r = fl(fl(m x / n)) with m x / n as d1*x/d2 left to right, m x stored, then / n: same as mx
                rr2 = st(LD(st(LD(m) * LD(x))) / LD(n)) if side1 else st(LD(n) / LD(st(LD(m) * LD(x))))
            else:
                rr2 = s3_forms.r_of(x, m, n, side1, rf)
            v = -LD(h) * LD(R.F.TD.log1p_xl(rr2))
        elif nm == "HS":  # -h LN(fl(1 + r)) (the log branch at every r)
            v = -LD(h) * LD(R.F.LN(st(LD(1) + LD(r))))
        elif nm.startswith("Hv_"):  # -h * log1p_var(r, variant), extended product
            v = -LD(h) * LD(log1p_var(r, nm[3:]))
        elif nm in ("Hcr", "Hcrs"):  # -h * (the correctly rounded double of log1p(stored r)); 's': product stored
            Lc = LD(float(mpmath.log1p(mpmath.mpf(r))))
            v = -sp(h, Lc) if nm == "Hcrs" else -LD(h) * Lc
        elif nm == "HxS":  # -h * Excel's series with its last ops (r + s, times 2) kept extended
            v = -LD(h) * series_ext(r)
        elif nm.startswith("q") and "_" in nm:  # q<coef>_<var>: -fl(coef * var), coef in a, b, h, am1, bm1, ap1, bp1,
            cn, vn = nm[1:].split("_")  # hm1, hm2, 1, 2;  var in L, Lx (LN x), Lr (LN r)
            cv = {"a": a, "b": b, "h": h, "am1": a - 1, "bm1": b - 1, "ap1": a + 1, "bp1": b + 1, "hm1": h - 1,
                  "hm2": h - 2, "1": 1.0, "2": 2.0, "o": b if side1 else a, "w": a if side1 else b,
                  "om1": (b if side1 else a) - 1, "wm1": (a if side1 else b) - 1, "ah": a / 2, "bh": b / 2, "hh": h / 2}[cn]
            vv = _var(vn, x, m, n, side1, r, L, Lx)
            v = -sp(cv, vv)
        elif nm.startswith("e") and "_" in nm:  # e<coef>_<var>: -coef * var extended
            cn, vn = nm[1:].split("_")
            cv = {"a": a, "b": b, "h": h, "am1": a - 1, "bm1": b - 1, "ap1": a + 1, "bp1": b + 1, "hm1": h - 1,
                  "hm2": h - 2, "1": 1.0, "2": 2.0, "o": b if side1 else a, "w": a if side1 else b,
                  "om1": (b if side1 else a) - 1, "wm1": (a if side1 else b) - 1, "ah": a / 2, "bh": b / 2, "hh": h / 2}[cn]
            vv = _var(vn, x, m, n, side1, r, L, Lx)
            v = -LD(cv) * vv
        elif nm in ("lw", "lo", "Sw", "So", "Sws", "Sos", "lws", "los", "Qo", "Qw", "Qos", "Qws", "Uo", "Uw"):
            # w = the side's own shape (a side 1, b side 2), o = the other; l: Excel log1p(r) (series),
            # S: LN(fl(1 + r)); Q: LN of the stored beta-side variable (o: fl(N_o / fl(n + mx)), N_o = n side 1 |
            # mx side 2; w: fl(N_w / fl(n + mx))); U: LN(fl(n + mx)) - LN(N_o) (difference of stored logs)
            # trailing 's': the product stored.  All enter as -coef * ln(1 + r) would.
            w, o = (a, b) if side1 else (b, a)
            cf = w if nm[1] == "w" else o
            mx = st(LD(m) * LD(x))
            den = st(LD(n) + LD(mx))
            No, Nw = (n, mx) if side1 else (mx, n)
            if nm[0] == "l":
                lv = L
            elif nm[0] == "S":
                lv = LD(R.F.LN(st(LD(1) + LD(r))))
            elif nm[0] == "Q":
                lv = -LD(R.F.LN(st(LD(No) / LD(den))))
            else:
                lv = LD(R.F.LN(den)) - LD(R.F.LN(No))
            v = -sp(cf, lv) if nm.endswith("s") else -LD(cf) * lv
        elif nm in ("Ht", "Hts", "Hs2", "Hs2s", "Ht2", "Ht2s"):  # -h*2t, -h*2s pieces of -h L (s: stored)
            tt, ss = series_parts(r)
            base_ = {"Ht": tt, "Hs2": ss, "Ht2": tt}[nm.rstrip("s") if nm != "Hs2s" else "Hs2"]
            v = -sp(2 * h, LD(base_)) if nm.endswith("s") and nm != "Hs2" else -LD(2 * h) * LD(base_)
        elif nm == "P":
            v = LD(st(LD(float(P))))
        elif nm == "Pe":  # the side's power product kept extended
            v = LD(a) * Lx if side1 else -LD(b) * Lx
        elif nm == "Pa":
            v = sp(a, Lx)
        elif nm == "Pb":
            v = -sp(b, Lx)
        elif nm == "Ph":
            v = -sp(h, Lx)
        elif nm == "Pa1":
            v = sp(a - 1, Lx)
        elif nm == "Pb1":
            v = -sp(b + 1, Lx)
        elif nm == "Ph1":
            v = -sp(h - 1, Lx)
        elif nm == "X":
            v = -Lx
        elif nm == "Lr":  # the power through r: side 1 +fl(a LN r), side 2 +fl(b LN r)
            v = sp(a if side1 else b, LD(R.F.LN(r)))
        elif nm == "Lmx":  # side 1 +fl(a LN mx), side 2 -fl(b LN mx)
            mx = st(LD(m) * LD(x))
            v = sp(a, LD(R.F.LN(mx))) if side1 else -sp(b, LD(R.F.LN(mx)))
        elif nm == "eL":  # Excel's LN(x) error (units) as a probe regressor is not a term; placeholder
            raise ValueError(nm)
        else:
            raise ValueError(nm)
        t[nm] = v
    return t


def prepare(rs, m, n, chain: str):
    toks = parse(chain)
    names = [nm for _, nm, _ in toks]
    T = [cell_terms(q["x"], m, n, names) for q in rs]
    arr = {nm: np.array([t[nm] for t in T], dtype=LD) for nm in names if nm not in ("C", "D")}
    hasX = "X" in names
    meas = [mpmath.log(mpmath.mpf(q["excel"])) + (0 if hasX else mpmath.log(mpmath.mpf(q["x"]))) for q in rs]
    # constant start: exact ln(f x) (or ln f with X) at cell 0 minus the computed terms there
    q0 = rs[0]
    ex0 = mpmath.log(q0["exact"]) + (0 if hasX else mpmath.log(mpmath.mpf(q0["x"])))
    K0 = ex0 - sum(_mp(v[0]) for v in arr.values())
    return toks, arr, meas, K0


def _mp(v):
    p, q = LD(v).as_integer_ratio()
    return mpmath.mpf(p) / q


def evaluate(toks, arr, C):
    acc = None
    for neg, nm, stv in toks:
        v = C if nm == "C" else arr[nm][None, :]
        if neg:
            v = -v
        acc = v if acc is None else acc + v
        if stv:
            acc = acc.astype(np.float64).astype(LD)
    return acc


def scan(rs, m, n, chain: str, span=320.0, step=0.25, ret=False):
    toks, arr, meas, K0 = prepare(rs, m, n, chain)
    Y0 = meas[0]
    Y = np.array([float((v - Y0) * R.U) for v in meas])
    ds = np.arange(-span, span, step)
    # C on an exact lattice (multiples of STEP units), so exact ties of a stored sum are reachable
    Kh = LD(float(K0))
    C = (Kh + ds.astype(LD) * LD(2.0) ** -53)[:, None]
    Ym = R.units_rel(evaluate(toks, arr, C), Y0)
    res = Y[None, :] - Ym
    res = res - np.median(res, axis=1)[:, None]
    sc = np.sum(np.abs(res) <= TOL, axis=1)
    i = int(np.argmax(sc))
    if ret:
        return int(sc[i]), float(ds[i]), res[i]
    return int(sc[i]), float(ds[i])


def evaluate2(toks, arr, C, D):
    """As evaluate, with a second free constant 'D'."""
    acc = None
    for neg, nm, stv in toks:
        v = C if nm == "C" else (D if nm == "D" else arr[nm][None, :])
        if neg:
            v = -v
        acc = v if acc is None else acc + v
        if stv:
            acc = acc.astype(np.float64).astype(LD)
    return acc


def scan2(rs, m, n, chain: str, D0, spanC=320.0, spanD=64.0, stepC=0.25, stepD=0.25, ret=False):
    """Two free constants: D near D0 (an mpf: the natural size of the piece that meets the x terms first), C near
    K0 - D0.  Returns (score, dC, dD[, res])."""
    toks, arr, meas, K0 = prepare(rs, m, n, chain)
    Y0 = meas[0]
    Y = np.array([float((v - Y0) * R.U) for v in meas])
    Dh = LD(float(D0))
    Ch = LD(float(K0 - mpmath.mpf(float(D0))))
    dsC = np.arange(-spanC, spanC, stepC)
    best = (-1, 0.0, 0.0, None)
    for dD in np.arange(-spanD, spanD, stepD):
        D = Dh + LD(dD) * LD(2.0) ** -53
        C = (Ch + dsC.astype(LD) * LD(2.0) ** -53)[:, None]
        Ym = R.units_rel(evaluate2(toks, arr, C, D), Y0)
        res = Y[None, :] - Ym
        res = res - np.median(res, axis=1)[:, None]
        sc = np.sum(np.abs(res) <= TOL, axis=1)
        i = int(np.argmax(sc))
        if sc[i] > best[0]:
            best = (int(sc[i]), float(dsC[i]), float(dD), res[i])
    return best if ret else best[:3]


def loader_split(m, n, side1):
    """(A, B): K' = A + B with A = (h - 1/2) ln(h/o) (o = b side 1, a side 2), B the rest (1/2 ln(w/2pi) +
    stirlerr terms), exact mpf."""
    a, b = mpmath.mpf(m) / 2, mpmath.mpf(n) / 2
    h = a + b
    o = b if side1 else a
    A = (h - mpmath.mpf(1) / 2) * mpmath.log(h / o)
    return A, R.s3_order.kprime(m, n, side1) - A


def series_parts(r: float):
    """(t, s) of Excel's 14-term atanh series: log1p(r) = 2 (t + s), t = r / (r + 2) stored, s the stored sum."""
    import excel_log1p as XLP

    X = XLP.X
    t = X("/", r, r + 2.0)
    t2 = X("*", t, t)
    p = t
    s = None
    for k in range(1, 15):
        p = X("*", p, t2)
        tm = X("*", p, XLP.CS[k])
        s = tm if s is None else X("+", s, tm)
    return t, s


def series_ext(r: float):
    """Excel's 14-term atanh series (r2-log1p's ops) with the final 2 (t + s) kept in extended."""
    t, s_ = series_parts(r)
    return LD(2) * (LD(t) + LD(s_))


def _var(vn, x, m, n, side1, r, L, Lx):
    """Variables for q/e terms: L (Excel log1p r), Lx (LN x), Lr (LN r), LR (LN of the larger ratio fl(n/mx) |
    fl(mx/n)), Lmx (LN(fl(m x))), L1R (LN(fl(1 + R))), LxL (fl(LN x - L)), Lq (LN(fl(N_o/fl(n + mx))))."""
    mx = st(LD(m) * LD(x))
    if vn == "L":
        return L
    if vn == "Lx":
        return Lx
    if vn == "Lr":
        return LD(R.F.LN(r))
    Rg = st(LD(n) / LD(mx)) if side1 else st(LD(mx) / LD(n))
    if vn == "LR":
        return LD(R.F.LN(Rg))
    if vn == "Lmx":
        return LD(R.F.LN(mx))
    if vn == "L1R":
        return LD(R.F.LN(st(LD(1) + LD(Rg))))
    if vn == "LxL":
        return LD(st(Lx - L))
    if vn == "Lq":
        den = st(LD(n) + LD(mx))
        return LD(R.F.LN(st(LD(n if side1 else mx) / LD(den))))
    raise ValueError(vn)


# ====================================================================== log1p variants (formerly lvar.py)
# Variants of Excel's ln(1 + r) for 0 < r <= 0.375, each op an x87 stored op (r2-log1p's x87), returned as a double.
#   base   r2-log1p's model: t = r/(r+2), t2 = t t, p = t t2^k by a running product, terms RN(1/(2k+1)) p summed largest
#          first (k = 1..14), 2 (t + s)
#   rev    the same terms summed smallest first
#   horner s = t (t2 (c1 + t2 (c2 + ...))) Horner in t2 from the top, then 2 (t + s)
#   hornerB s = (t t2) * Horner(c1..c14 in t2)
#   two    2t + 2s (each doubled, summed)
#   tinv   t = r * fl(1/(r+2))
#   ext    base with the final 2 (t + s) kept extended (returned as longdouble)
#   toms   TOMS 708 alnrel's (3,3) rational in t (w1-density alnrel_x87)
#   N<k>   base with k terms (e.g. N8, N10, N16)
# 
import functools  # noqa: E402

import excel_log1p as XLP  # noqa: E402  (r2-log1p, on flib's path)

X = XLP.X
CS = [1.0 / (2 * k + 1) for k in range(40)]
TP = (-0.129418923021993e01, 0.405303492862024e00, -0.178874546012214e-01)
TQ = (-0.162752256355323e01, 0.747811014037616e00, -0.845104217945565e-01)


def _terms(t, t2, N):
    p = t
    out = []
    for k in range(1, N + 1):
        p = X("*", p, t2)
        out.append(X("*", p, CS[k]))
    return out


@functools.lru_cache(maxsize=None)
def log1p_var(r: float, var: str = "base") -> float:
    """Excel-style ln(1 + r) for 0 < r <= 0.375 by variant (see the section comment above)."""
    if var == "toms":  # TOMS 708 alnrel's coefficients (R toms708.c), stored x87 ops
        t = X("/", r, X("+", r, 2.0))
        t2 = X("*", t, t)
        num = X("+", X("*", X("+", X("*", X("+", X("*", TP[2], t2), TP[1]), t2), TP[0]), t2), 1.0)
        den = X("+", X("*", X("+", X("*", X("+", X("*", TQ[2], t2), TQ[1]), t2), TQ[0]), t2), 1.0)
        return X("*", X("*", t, 2.0), X("/", num, den))
    N = 14
    if var.startswith("N"):
        N = int(var[1:])
    if var == "tinv":
        t = X("*", r, X("/", 1.0, r + 2.0))
    else:
        t = X("/", r, X("+", r, 2.0))
    t2 = X("*", t, t)
    if var in ("horner", "hornerB"):
        acc = CS[N]
        for k in range(N - 1, 0, -1):
            acc = X("+", X("*", acc, t2), CS[k])
        if var == "horner":
            s = X("*", t, X("*", t2, acc))
        else:
            s = X("*", X("*", t, t2), acc)
        return X("*", 2.0, X("+", t, s))
    tm = _terms(t, t2, N)
    if var == "rev":
        tm = tm[::-1]
    s = tm[0]
    for v in tm[1:]:
        s = X("+", s, v)
    if var == "two":
        return X("+", X("*", 2.0, t), X("*", 2.0, s))
    return X("*", 2.0, X("+", t, s))
