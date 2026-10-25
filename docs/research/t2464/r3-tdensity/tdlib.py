"""T-2513 r3-tdensity: shared code for Excel's Student t density T.DIST(t, n, FALSE).

* Data: r2-tdist's `tlib.load(("pdf",))` rows of ../cells.tsv (imported), plus this directory's probes
  (eval/suites/probes-numeric/cases/t2513-r3-tdensity-*.case.json, values from the oracle sidecars).
* Arithmetic: numpy longdouble is the x87 64-bit register, `st()` stores to a double (fl53(fl64(.)), the
  playbook's stored x87 op).  Excel LN and EXP (model V6 on a double or extended argument) come from
  r3-loader-bd0's `r3lib` (imported, not copied).
* Models are written as F(C) for a free per-df constant C (the shape part, which in Loader's dt is
  -bd0(n/2, (n+1)/2) + stirlerr((n+1)/2) - stirlerr(n/2)); `interval()` inverts a monotone F per cell and
  r3lib.best_overlap picks the C shared by most cells.  `exact_*` give the exact reference pieces (mpmath).
Run with /home/user/rustsheet/bindings/python/.venv/bin/python from this directory.
"""

from __future__ import annotations

import hashlib
import json
import math
import pickle
import sys
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
T = HERE.parent
for p in (T / "r3-loader-bd0", T / "r2-tdist", T / "r2-log1p", T):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))
import r3lib as R3  # noqa: E402

LDB = np.longdouble
st = R3.st
LN = R3.LN
LN64 = R3.LN64
exp_v6 = R3.exp_v6
best_overlap = R3.best_overlap
hits_at = R3.hits_at
CASES = Path("/home/user/rustsheet/eval/suites/probes-numeric/cases")
U53 = LDB(2.0) ** -53
TWO_PI = float(2 * mpmath.pi)
INV_SQRT_2PI = float(1 / mpmath.sqrt(2 * mpmath.pi))
LN_SQRT_2PI = float(mpmath.log(mpmath.sqrt(2 * mpmath.pi)))
SQRT_PI = float(mpmath.sqrt(mpmath.pi))


# ---------------------------------------------------------------- data
def hold(case: str, cell: str) -> bool:
    return int(hashlib.md5(repr((case, cell)).encode()).hexdigest(), 16) % 2 == 1


def load_old():
    """T.DIST(t, n, FALSE) rows from ../cells.tsv (r2-tdist's loader): t, n (truncated df), excel, case, cell."""
    import tlib

    return [r for r in tlib.load(("pdf",)) if r["t"] != 0 and r["excel"] != 0]


def load_probe(name: str):
    """Rows of probe t2513-r3-tdensity-<name>: meta in this directory (<name>.meta.json: cell -> [blk, t, n] and
    the inner-column map), values from the eval oracle sidecar.  Returns rows with every sheet value read back."""
    meta = json.loads((HERE / f"{name}.meta.json").read_text())
    side = CASES / f"t2513-r3-tdensity-{name}.oracle.json"
    if not side.exists():
        return []
    sh = json.loads(side.read_text())["sheets"]["Calc"]
    rows = []
    for row in meta["rows"]:
        r = dict(blk=row["blk"], t=row["t"], n=row["n"], row=row["row"], case=name)
        ok = True
        for col, key in meta["cols"].items():
            c = sh.get(f"{col}{row['row']}")
            if c is None or c.get("t") != "n":
                ok = False
                break
            r[key] = float(c["v"])
        if ok:
            r["cell"] = f"B{row['row']}"
            r["hold"] = hold(name, r["cell"])
            rows.append(r)
    return rows


# ---------------------------------------------------------------- exact references
_EXF = HERE / "exact_cache.pkl"
_EX: dict = pickle.loads(_EXF.read_bytes()) if _EXF.exists() else {}


def exact_pdf(t: float, n: float) -> float:
    k = ("pdf", abs(t), n)
    if k not in _EX:
        with mpmath.workdps(40):
            T_, N = mpmath.mpf(t), mpmath.mpf(n)
            lg = mpmath.loggamma((N + 1) / 2) - mpmath.loggamma(N / 2)
            _EX[k] = float(
                mpmath.exp(lg - (N + 1) / 2 * mpmath.log1p(T_ * T_ / N)) / mpmath.sqrt(N * mpmath.pi)
            )
    return _EX[k]


def exact_logpdf(t: float, n: float):
    with mpmath.workdps(40):
        T_, N = mpmath.mpf(t), mpmath.mpf(n)
        return (
            mpmath.loggamma((N + 1) / 2)
            - mpmath.loggamma(N / 2)
            - (N + 1) / 2 * mpmath.log1p(T_ * T_ / N)
            - mpmath.log(mpmath.sqrt(N * mpmath.pi))
        )


def save_exact():
    _EXF.write_bytes(pickle.dumps(_EX))


def stirlerr_exact(v):
    v = mpmath.mpf(v)
    return mpmath.loggamma(v + 1) - (v + 0.5) * mpmath.log(v) + v - mpmath.log(mpmath.sqrt(2 * mpmath.pi))


def bd0_exact(x, np_):
    x, np_ = mpmath.mpf(x), mpmath.mpf(np_)
    return x * mpmath.log(x / np_) + np_ - x


def loader_const_exact(n: float):
    """t = -bd0(n/2, (n+1)/2) + stirlerr((n+1)/2) - stirlerr(n/2), exact (mpmath, 40 digits)."""
    with mpmath.workdps(40):
        N = mpmath.mpf(n)
        return -bd0_exact(N / 2, (N + 1) / 2) + stirlerr_exact((N + 1) / 2) - stirlerr_exact(N / 2)


def ld_of(m) -> np.longdouble:
    return R3._ld_of(m)


# ---------------------------------------------------------------- scoring
def w15(m: float, e: float) -> bool:
    if m == e:
        return True
    if e == 0 or not math.isfinite(m):
        return False
    q = math.floor(math.log10(abs(e))) - 14
    return abs(m - e) <= 0.5 * 10.0**q * (1 + 1e-12)


def ulp_close(m: float, e: float, k: int = 4) -> bool:
    return abs(m - e) <= k * math.ulp(e)


def tally(pairs):
    """pairs (model, excel) -> (n, exact, w4, w15)."""
    n = ex = f4 = v = 0
    for m, e in pairs:
        n += 1
        ex += m == e
        f4 += ulp_close(m, e)
        v += w15(m, e)
    return n, ex, f4, v


def fmt(t) -> str:
    return f"{t[1]}/{t[2]}/{t[3]} of {t[0]}"


def interval(F, y: float, W: float = 64.0, tol: float = 2e-4):
    """Offsets u in [-W, W] (units 2^-53 of C) with F(u) == y for a non-decreasing F, as [(lo, hi)] or []."""
    return R3.interval(F, y, W, tol)


# ---------------------------------------------------------------- Loader's dt (R) with x87 knobs
def _R(v, mode):
    return LDB(st(v)) if mode == "x" else LDB(v)


def log1p_xl(a: float) -> float:
    """Excel's ln(1+a): r2-log1p's 14-term atanh series for |a| <= 0.375, else LN(fl(1+a))."""
    import excel_log1p as XL

    if -0.375 <= a <= 0.375:
        return XL.log1p_series(a)
    return LN(st(LDB(1) + LDB(a)))


def log_branch_l(xx, q, N, lnf: str, ycut: float = 0.5) -> np.longdouble:
    """ln(1 + x x/n) in the log branch, by form: 'ln' LN(fl(1+q)); 'l1p' Excel log1p(q); 'ln64' FYL2X unstored;
    'lny' -LN(y), y = fl(n/fl(n+xx)); 'l1mz' -LN(fl(1-z)), z = fl(xx/fl(n+xx)); 'l1pz' -log1p_xl(-z);
    'auto' -LN(y) for y >= ycut else LN(fl(1+q)); 'autoz' -log1p_xl(-z) for y >= ycut else LN(fl(1+q));
    'lnu' LN(fl(n+xx)) - LN(n) (stored difference)."""
    w = st(LDB(1) + q)
    u = LDB(st(N + xx))
    y = st(N / u)
    z = st(xx / u)
    if lnf == "auto":
        lnf = "lny" if y >= ycut else "ln"
    elif lnf == "autoe":
        lnf = "lnye" if y >= ycut else "ln"
    if lnf == "lnye":  # y = fl(n / (n + xx)) with n + xx kept extended
        return -LDB(LN(st(N / (N + xx))))
    if lnf == "l1pze":  # z = fl(xx / (n + xx)) extended denominator, -log1p_xl(-z)
        return -LDB(log1p_xl(-st(xx / (N + xx))))
    elif lnf == "autoz":
        lnf = "l1pz" if y >= ycut else "ln"
    if lnf == "ln":
        return LDB(LN(w))
    if lnf == "l1p":
        return LDB(log1p_xl(float(q)))
    if lnf == "ln64":
        return LN64(w)
    if lnf == "lny":
        return -LDB(LN(y))
    if lnf == "l1mz":
        return -LDB(LN(st(LDB(1) - LDB(z))))
    if lnf == "l1pz":
        return -LDB(log1p_xl(-z))
    if lnf == "lnu":
        return LDB(st(LDB(LN(float(u))) - LDB(LN(float(N)))))
    raise ValueError(lnf)


def loader_u(t: float, n: float, o: dict):
    """The t-dependent part of Loader's dt: returns (u, l_x2n, branch) as longdoubles.
    o: br 'xx' (x*x > 0.2 n, R 2.x) | 'q' (x2n > 0.2, R 3.x); lnf 'ln' LN(fl(1+q)) | 'l1p' Excel log1p (log branch);
    ua 'x'/'e' storage of u's ops; bd0 knobs for r3lib.bd0 (series branch); np 'x'/'e' (n + x x)/2 stored or not."""
    ua = o.get("ua", "x")
    X = LDB(t)
    N = LDB(n)
    xx = _R(X * X, o.get("xx", "x"))
    q = _R(xx / N, o.get("qa", "x"))
    br = o.get("br", "xx")
    if br == "xx":
        big = xx > _R(LDB(o.get("bcut", 0.2)) * N, "x")
    elif br == "none":  # always -bd0(n/2, (n + x x)/2) + x x/2 (bd0's own cut picks series or log form)
        big = False
    else:
        big = q > LDB(o.get("bcut", 0.2))
    if big:
        l = log_branch_l(xx, q, N, o.get("lnf", "ln"), o.get("ycut", 0.5))
        uf = o.get("uform", "ln_n_2")
        if uf == "ln_n_2":  # log(1 + x x / n) * n / 2
            u = _R(_R(l * N, ua) / 2, ua)
        elif uf == "n2":  # fl(n/2) * log(..)
            u = _R(LDB(st(N / 2)) * l, ua)
        else:  # n * (log(..)/2)
            u = _R(N * _R(l / 2, ua), ua)
        return u, l / 2, "l"
    np_ = _R(_R(N + xx, o.get("np", "x")) / 2, o.get("np", "x"))
    k = LDB(st(N / 2))
    b, _ = R3.bd0(k, np_, o.get("bd0", {}))
    u = _R(-b + _R(xx / 2, ua), ua)
    lq = None
    if o.get("den", "old") != "old":
        lq = LDB(log1p_xl(float(q))) / 2
    return u, lq, "s"


def loader_dt(t: float, n: float, C, o: dict) -> float:
    """Loader dt with shape constant C (longdouble): E = C - u; old R: EXP(E) / sqrt(2 pi (1 + q)); new R:
    EXP(E) * M_1_SQRT_2PI * EXP(-l_x2n)."""
    u, lq, _ = cache_u(t, n, o)
    E = _R(LDB(C) - u, o.get("E", "x"))
    ex = exp_v6(E, True)
    ex = _R(ex, o.get("ex", "x"))
    den = o.get("den", "old")
    if den == "old":
        N = LDB(n)
        X = LDB(t)
        xx = _R(X * X, o.get("dxx", o.get("xx", "x")))
        q = _R(xx / N, o.get("dq", o.get("qa", "x")))
        w = _R(LDB(1) + q, o.get("dw", o.get("fa", "x")))
        f = _R(LDB(TWO_PI) * w, o.get("fa", "x"))
        s = _R(np.sqrt(f), o.get("sa", "x"))
        return st(ex / s)
    if den == "sw":  # EXP(E) / sqrt(w) with 1/sqrt(2 pi) folded into C
        w = LDB(st(LDB(1) + LDB(st(LDB(st(LDB(t) * LDB(t))) / LDB(n)))))
        return st(ex / _R(np.sqrt(w), o.get("sa", "x")))
    if den in ("sw2", "sw3", "sw4", "sw5"):  # 1/sqrt(2 pi) applied separately, sqrt(w) stored
        w = LDB(st(LDB(1) + LDB(st(LDB(st(LDB(t) * LDB(t))) / LDB(n)))))
        sw = LDB(st(np.sqrt(w)))
        if den == "sw2":
            return st(LDB(st(ex * LDB(INV_SQRT_2PI))) / sw)
        if den == "sw3":
            return st(LDB(st(ex / sw)) * LDB(INV_SQRT_2PI))
        if den == "sw4":
            return st(ex / LDB(st(sw * LDB(float(mpmath.sqrt(2 * mpmath.pi))))))
        return st(LDB(st(ex / LDB(float(mpmath.sqrt(2 * mpmath.pi))))) / sw)
    if den == "swm":  # EXP(E) * fl(1/sqrt(w))
        w = LDB(st(LDB(1) + LDB(st(LDB(st(LDB(t) * LDB(t))) / LDB(n)))))
        return st(ex * LDB(st(LDB(1) / LDB(st(np.sqrt(w))))))
    if den == "sy":  # EXP(E) * sqrt(y)
        p = pieces(t, n)
        return st(ex * _R(np.sqrt(p["y"]), o.get("sa", "x")))
    # new R: exp(t-u) * M_1_SQRT_2PI * I_sqrt_, I_sqrt_ = exp(-l_x2n)
    I = _R(exp_v6(_R(-lq, "x"), True), "x")
    return st(_R(ex * LDB(INV_SQRT_2PI), o.get("fa", "x")) * I)


_UC: dict = {}


def cache_u(t, n, o):
    key = (t, n, json.dumps(o, sort_keys=True))
    r = _UC.get(key)
    if r is None:
        r = _UC[key] = loader_u(t, n, o)
    return r


# ---------------------------------------------------------------- generic free-constant structures
def pieces(t: float, n: float) -> dict:
    """Worksheet intermediates as x87 stored doubles (longdouble holding a double)."""
    X, N = LDB(t), LDB(n)
    tt = LDB(st(X * X))
    q = LDB(st(tt / N))
    w = LDB(st(LDB(1) + q))
    u = LDB(st(N + tt))
    y = LDB(st(N / u))
    z = LDB(st(tt / u))
    h = LDB((n + 1) / 2)
    return dict(t=X, n=N, tt=tt, q=q, w=w, u=u, y=y, z=z, h=h)


def struct_out(name: str, p: dict, C) -> float:
    """Density of a structure with constant C (longdouble).  Names: <log>_<chain>
    log: 'L1' LN(w), 'Lp' Excel log1p(q), 'Lu' LN(u) (C absorbs n^(n/2)... ), 'Ly' -LN(y), 'Lz' -log1p_xl(-z),
         'L64' FYL2X(w) unstored.
    chain: 'Es' E = fl(C - fl(h L)) stored; 'Ee' E = C - h L extended; 'Ep' E = C - fl(h L) extended sum;
           'K' out = fl(C * EXP(-fl(h L))) (C a value); 'Ke' out = fl(C * EXP_ext(-h L));
           'Hs' E = fl(fl(C - fl(n/2 L)) - L/2) ; 'Q' out = fl(EXP(fl(C - fl(n/2 L))) / sqrt(w)) (Loader-like)."""
    lg, ch = name.split("_")
    L = p.get("_L_" + lg)
    if L is None:
        if lg == "L1":
            L = LDB(LN(float(p["w"])))
        elif lg == "Lp":
            L = LDB(log1p_xl(float(p["q"])))
        elif lg == "Lu":
            L = LDB(LN(float(p["u"])))
        elif lg == "Ly":
            L = -LDB(LN(float(p["y"])))
        elif lg == "Lz":
            L = -LDB(log1p_xl(-float(p["z"])))
        elif lg == "L64":
            L = LN64(float(p["w"]))
        p["_L_" + lg] = L
    h = p["h"]
    if ch == "Es":
        return exp_v6(LDB(st(C - LDB(st(h * L)))))
    if ch == "Ee":
        return exp_v6(C - h * L)
    if ch == "Ep":
        return exp_v6(C - LDB(st(h * L)))
    if ch == "K":
        return st(C * LDB(exp_v6(LDB(st(-h * L)))))
    if ch == "Ke":
        return st(C * exp_v6(-h * L, True))
    hn = p["n"] / 2
    if ch == "Hs":
        return exp_v6(LDB(st(LDB(st(C - LDB(st(hn * L)))) - L / 2)))
    if ch == "Q":
        return st(LDB(exp_v6(LDB(st(C - LDB(st(hn * L)))))) / LDB(st(np.sqrt(p["w"]))))
    if ch == "Qe":
        return st(exp_v6(C - hn * L, True) / np.sqrt(p["w"]))
    raise ValueError(name)


def struct_C0(name: str, n: float):
    """Exact constant for a structure (log K, or K for value chains), mpmath."""
    lg, ch = name.split("_")
    with mpmath.workdps(40):
        N = mpmath.mpf(n)
        lk = mpmath.loggamma((N + 1) / 2) - mpmath.loggamma(N / 2) - mpmath.log(N * mpmath.pi) / 2
        if lg == "Lu":
            lk += (N + 1) / 2 * mpmath.log(N)
        if ch in ("K", "Ke"):
            return mpmath.exp(lk)
        if ch in ("Q", "Qe"):
            return lk
        return lk


# ---------------------------------------------------------------- Excel GAMMALN / GAMMA (harness, cached here)
_GLF = HERE / "gl_cache.pkl"
_GL: dict = pickle.loads(_GLF.read_bytes()) if _GLF.exists() else {}


def xl_fn(name: str, vs) -> list:
    """Excel's GAMMALN / GAMMA / EXP / LN of doubles, batched through ../excelmath (cached in gl_cache.pkl)."""
    import excelmath

    need = sorted({float(v) for v in vs if (name, float(v)) not in _GL})
    if need:
        for v, r in zip(need, excelmath.excel(name, need)):
            _GL[(name, v)] = r
        _GLF.write_bytes(pickle.dumps(_GL))
    return [_GL[(name, float(v))] for v in vs]


def GL(v: float) -> float:
    return xl_fn("GAMMALN", [v])[0]


# ---------------------------------------------------------------- M1 beta density at (y, n/2, 1/2) with z complement
def lnb_gl(a: float, b: float) -> np.longdouble:
    """M1's ln B for min < 1, max <= 8: fl(GL(min) + fl(GL(max) - GL(a+b))) (statistics.md 'Student t prefactor')."""
    lo, hi = min(a, b), max(a, b)
    return LDB(st(LDB(GL(lo)) + LDB(st(LDB(GL(hi)) - LDB(GL(st(LDB(a) + LDB(b))))))))


def m1_brcomp(p: dict, o: dict) -> np.longdouble:
    """x^a y^b / B with x = y_T, y = z_T, a = n/2, b = 1/2 in M1 style: tx = fl(fl(a-1) lnx) ... E extended,
    P = EXP(E) (the density at x), returned as fl(P * fl(x*y)) (w4's pdf_xy) unless o['pdfonly']."""
    a, b = float(p["n"]) / 2, 0.5
    y, z = float(p["y"]), float(p["z"])
    sw = o.get("sw", "toms")
    if sw == "toms":
        if y <= 0.375:
            lx, ly = LN(y), log1p_xl(-y)
        elif z > 0.375:
            lx, ly = LN(y), LN(z)
        else:
            lx, ly = log1p_xl(-z), LN(z)
    else:  # M1 style with the explicit complement: LN(x); ln(1-x) = log1p(-x) for x <= 0.375 else LN(z)
        lx = LN(y)
        ly = log1p_xl(-y) if y <= 0.375 else LN(z)
    lnb = o.get("lnb")
    lnb = lnb_gl(a, b) if lnb is None else lnb
    tx = LDB(st(LDB(st(LDB(a) - 1)) * LDB(lx)))
    ty = LDB(st(LDB(-0.5) * LDB(ly)))
    E = tx + (ty - lnb) if o.get("E", "e") == "e" else LDB(st(tx + LDB(st(ty - lnb))))
    P = exp_v6(E, True)
    P = LDB(st(P))
    if o.get("pdfonly"):
        return P
    return LDB(st(P * LDB(st(LDB(y) * LDB(z)))))


# ---------------------------------------------------------------- the frozen round-3 model
LT1 = {"bcut": 1.0, "bd0": {"v": "x", "ph": 1}, "den": "sw3"}
# Free per-df constants (offset from the exact Loader constant, units of 2^-53 absolute), fitted on the in-sample
# probes a1/a2 by s17_final.py; other dfs use the exact constant.
LT1_OFF: dict = {}


def lt1(t: float, n: float, off: float | None = None) -> float:
    """T.DIST(t, n, FALSE), model LT1 (see RESULTS.md): Loader's dt with Excel pieces.  n >= 3."""
    if off is None:
        off = LT1_OFF.get(n, 0.0)
    C = ld_of(loader_const_exact(n)) + LDB(off) * U53
    return loader_dt(abs(t), n, C, LT1)


# ---------------------------------------------------------------- LT1e: LT1 with the stirlerr-table constant
def stirlerr_e() -> dict:
    """r3-loader-const's fitted per-half-integer stirlerr corrections e(v) (units), read from its grid_e.tsv (that
    worker's BETA.DIST grid fit, gauge alpha v + gamma free)."""
    import csv

    with open(T / "r3-loader-const" / "grid_e.tsv") as fh:
        return {float(r["v"]): float(r["e_units"]) for r in csv.DictReader(fh, delimiter="\t")}


def lt1e_offsets(c0: float, ns) -> dict:
    """Offset of the T constant for each n: e((n+1)/2) - e(n/2) + c0 (c0 = alpha/2 gauge, fitted in-sample)."""
    e = stirlerr_e()
    return {float(n): e[(n + 1) / 2] - e[n / 2] + c0 for n in ns if (n + 1) / 2 in e and n / 2 in e}


def fit_free_C(rows, n: float, o: dict, target=None, W: float = 48.0):
    """Free-constant fit of loader_dt(o) on rows of one df: per-cell offset intervals (units of 2^-53 from the exact
    Loader constant, minus ln sqrt(2 pi) for the folded denominators) and their deepest overlap.  `target` maps
    id(row) to a synthetic value (positive controls).  Returns (count, offset, intervals, C0)."""
    C0 = loader_const_exact(n)
    if o.get("den", "old") in ("sw", "swm", "sy"):
        C0 -= mpmath.log(mpmath.sqrt(2 * mpmath.pi))
    C0 = ld_of(C0)
    ivs = [interval(lambda u, r=r: loader_dt(r["t"], n, C0 + LDB(u) * U53, o),
                    r["excel"] if target is None else target[id(r)], W=W) for r in rows]
    nb, pos = best_overlap(ivs)
    return nb, pos, ivs, C0
