"""T-2464 r2-bpser-small: shared code. Imports r2-bratio's TOMS 708 port (bratio_excel) and scorer (score_lib) and
w2-forensics' exact literal encoder; adds (1) a tracing kernel that records which bpser/apser/fpser call fired and its
TOMS sub-branch, (2) x87 ports of TOMS bpser with Excel's EXP/LN/GAMMALN and every prefactor branch, (3) the probe
data loader. Run with /home/user/rustsheet/bindings/python/.venv/bin/python.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import math
import pickle
import sys
from dataclasses import dataclass, replace
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
sys.path.insert(0, str(T2464 / "r2-bratio"))
import bratio_excel as bx  # noqa: E402
import score_lib as sl  # noqa: E402

_spec = importlib.util.spec_from_file_location("w2probes_bs", T2464 / "w2-forensics" / "make_probes.py")
w2p = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(w2p)

sx = bx.sx
EXP, EXPX, LN, GAMMALN = bx.EXP, bx.EXPX, bx.LN, bx.GAMMALN
mpf = mpmath.mpf
fl64, fl53 = bx.em.fl64, bx.em.fl53
EPS = bx.Kernel.EPS
CASES = w2p.CASES
PROBES = ("shapes", "runs", "tiny")  # BETA.DIST probes; "gl" is GAMMALN


def sub_branch(fn: str, a: float, b: float) -> str:
    """TOMS sub-branch inside bpser(a, b, ...): by a0 = min, b0 = max."""
    if fn != "bpser":
        return fn
    a0, b0 = min(a, b), max(a, b)
    if a0 >= 1.0:
        return "bpser:a0>=1"
    if b0 <= 1.0:
        return "bpser:b0<=1"
    if b0 < 8.0:
        return "bpser:1<b0<8"
    return "bpser:b0>=8"


# ---------------------------------------------------------------------------------------------- models
@dataclass(frozen=True)
class V:
    """A bpser variant.
    pre    "w3": EXP(a LN x - L) with w3's L (lnb_w3); "toms": TOMS's prefactor branches (x87 stored, Excel EXP/LN)
    ser    "w3": s = fl(1/a) + sum c/(a+n), I = P*s (P without the 1/a); "toms": I = ans*(1 + a*s), s = sum c/(a+n)
    z      "z64"/"z53": w3's EXP argument kept extended / stored (pre="w3" only)
    pw     TOMS b0 <= 1: x^a as "explna" EXP(fl(a LN x)), or "explnx" EXP of the extended a*LN x
    tomsz  TOMS 1<b0<8 and b0>=8: EXP argument "z53" (z stored) or "z64"
    """

    pre: str = "w3"
    ser: str = "w3"
    z: str = "z64"
    pw: str = "explna"
    tomsz: str = "z53"
    sub: tuple = ()  # restrict pre="toms" to these sub-branches (others use w3); () = all
    nofpser: bool = False  # Excel has no fpser: bratio falls through to w = bpser(a0, b0, x0)
    noapser: bool = False  # Excel has no apser: bratio falls through (x0 >= 0.3: bpser(b0,a0,y0), else bup+bgrat/bgrat)
    alx: str = "53"  # z's a*LN(x): "53" stored, "64" extended product, "ln64" extended LN and product
    eext: bool = False  # EXP's result kept in extended (x87 register, w*2^n unstored) for the product with s
    lg: str = ""  # "L53[/algdiv64|algdiv53|glalgdiv]": see Model.lnb_w3; otherwise override w3's lnGamma(v < 1): "toms53" = fl53(gamln1(v) - LN v), "gl", "toms", "acc"; "" = cfg


class Model(bx.Kernel):
    """bx.Kernel with bpser replaced by a variant V, recording each series call's (fn, a, b, x)."""

    def __init__(self, cfg, v: V):
        super().__init__(cfg)
        self.v = v
        self.trace = []

    def fpser(self, a, b, x, eps):
        if self.v.nofpser:  # the rest of bratio's min <= 1 dispatch sends b0 tiny to w = bpser(a0, b0, x0)
            return self.bpser(a, b, x, eps)
        self.trace.append(("fpser", a, b, x))
        return super().fpser(a, b, x, eps)

    def apser(self, a, b, x, eps):
        if self.v.noapser:
            # bratio without apser, for a0 < 1e-15 (so pow(x0 b0, a0) > 0.7 and x0^a0 > 0.9): returns w1.
            # y0 = 1 - x0 is exact for BETA.DIST's swapped cells (x > 0.5, x0 = fl(1 - x)).
            y = 0.5 - x + 0.5
            if x >= self.c.x29:
                return self.bpser(b, a, y, eps)
            if b > 15.0:
                return self.bgrat(b, a, y, x, 0.0, 15.0 * eps)
            w1 = self.bup(b, a, y, x, 20, eps)
            return self.bgrat(b + 20, a, y, x, w1, 15.0 * eps)
        self.trace.append(("apser", a, b, x))
        return super().apser(a, b, x, eps)

    def lnb_w3(self, a, b):
        lo, hi = min(a, b), max(a, b)
        if self.v.lg.startswith("GL53") and lo < 1.0:
            # zforms.py: L = fl(GL(a0) + fl(GL(b0) - GL(a0 + b0))) all Excel GAMMALN, stored, smaller shape first
            # (TOMS betaln's grouping); b0 >= 8: fl(GL(a0) + algdiv(a0, b0)) ("/glalgdiv") or the accurate proxy
            if hi < 8.0:
                return mpf(sx("+", GAMMALN(lo), sx("-", GAMMALN(hi), GAMMALN(sx("+", lo, hi)))))
            if self.v.lg.endswith("/glalgdiv"):
                return mpf(sx("+", GAMMALN(lo), self.algdiv(lo, hi)))
            return super().lnb_w3(a, b)
        if self.v.lg.startswith("L53") and lo < 1.0:
            # TOMS betaln for a0 < 1 in doubles, gamln(a0) = gamln1(a0) - LN(a0) (also for 0.8 < a0 < 1), Excel
            # GAMMALN for the arguments >= 1 (lscan/lcover: the L that fits the 1 < b0 < 8 runs exactly)
            g0 = sx("-", self.gamln1(lo), LN(lo))
            if hi < 8.0:
                return mpf(sx("+", g0, sx("-", GAMMALN(hi), GAMMALN(sx("+", lo, hi)))))
            big = self.v.lg.split("/")[1] if "/" in self.v.lg else "acc"
            if big == "algdiv64":
                return fl64(mpf(g0) + mpf(self.algdiv(lo, hi)))
            if big == "algdiv53":
                return mpf(sx("+", g0, self.algdiv(lo, hi)))
            if big == "glalgdiv":
                return mpf(sx("+", GAMMALN(lo), self.algdiv(lo, hi)))
            return super().lnb_w3(a, b)
        if not self.v.lg or self.v.lg[:3] in ("L53", "GL5") or max(a, b) > 8.0:
            return super().lnb_w3(a, b)
        if self.v.lg != "toms53":
            return bx.Kernel(replace(self.c, w3_lg=self.v.lg)).lnb_w3(a, b)

        def lg(v):
            return mpf(GAMMALN(v)) if v >= 1.0 else mpf(sx("-", self.gamln1(v), LN(v)))

        return fl64(lg(a) + fl64(lg(b) - lg(sx("+", a, b))))

    # -- the series ------------------------------------------------------------------------------------------
    @staticmethod
    def series_toms(a, b, x, eps):
        """TOMS bpser's sum, x87 stored: s = sum c/(a+n), c = c*((0.5 - b/n) + 0.5)*x, tol = eps/a."""
        tol, n, s, c = eps / a, 0.0, 0.0, 1.0
        while True:
            n += 1.0
            c = sx("*", sx("*", c, sx("+", sx("-", 0.5, sx("/", b, n)), 0.5)), x)
            w = sx("/", c, sx("+", a, n))
            s = sx("+", s, w)
            if not (n < 1e7 and abs(w) > tol):
                break
        return s

    def prefactor_toms(self, a, b, x):
        """TOMS bpser's ans (= x^a / (a B(a,b))), every op stored x87, with Excel's EXP/LN and TOMS gam1/gamln1/algdiv
        (themselves double code; Python floats = SSE doubles)."""
        v = self.v
        a0, b0 = min(a, b), max(a, b)
        if a0 >= 1.0:
            return sx("/", EXP(sx("-", sx("*", a, LN(x)), self.betaln(a, b))), a)
        if b0 >= 8.0:
            u = self.gamln1(a0) + self.algdiv(a0, b0)
            zz = mpf(sx("*", a, LN(x))) - mpf(u)
            e = EXP(fl53(zz)) if v.tomsz == "z53" else EXPX(fl64(zz))
            return sx("*", sx("/", a0, a), e)
        if b0 <= 1.0:
            if v.pw == "explna":
                ans = EXP(sx("*", a, LN(x)))
            else:
                ans = EXPX(fl64(mpf(a) * mpf(LN(x))))
            if ans == 0.0:
                return 0.0
            apb = sx("+", a, b)
            if apb > 1.0:
                z = sx("/", sx("+", self.gam1(sx("-", apb, 1.0)), 1.0), apb)
            else:
                z = sx("+", self.gam1(apb), 1.0)
            c = sx("/", sx("*", sx("+", self.gam1(a), 1.0), sx("+", self.gam1(b), 1.0)), z)
            return sx("*", ans, sx("*", c, sx("/", b, apb)))
        u = self.gamln1(a0)
        m = int(b0 - 1.0)
        if m >= 1:
            c = 1.0
            for _ in range(m):
                b0 = b0 - 1.0
                c = sx("*", c, sx("/", b0, sx("+", a0, b0)))
            u = sx("+", LN(c), u)
        zz = mpf(sx("*", a, LN(x))) - mpf(u)
        b0 = b0 - 1.0
        apb = sx("+", a0, b0)
        if apb > 1.0:
            t = sx("/", sx("+", self.gam1(sx("-", apb, 1.0)), 1.0), apb)
        else:
            t = sx("+", self.gam1(apb), 1.0)
        e = EXP(fl53(zz)) if v.tomsz == "z53" else EXPX(fl64(zz))
        return sx("/", sx("*", sx("*", e, sx("/", a0, a)), sx("+", self.gam1(b0), 1.0)), t)

    def bpser(self, a, b, x, eps):
        if x == 0.0:
            return 0.0
        self.trace.append(("bpser", a, b, x))
        v = self.v
        use_toms = v.pre == "toms" and (not v.sub or sub_branch("bpser", a, b) in v.sub)
        if not use_toms:
            if v.ser == "w3":
                return self.bpser_w3(a, b, x)
            # w3 prefactor (includes 1/a) times TOMS's (1 + a s)
            p = self.w3_pre(a, b, x)
            ans = sx("/", p, a)
            s = self.series_toms(a, b, x, eps)
            return sx("*", ans, sx("+", sx("*", a, s), 1.0))
        ans = self.prefactor_toms(a, b, x)
        if ans == 0.0 or a <= eps * 0.1:
            return ans
        if v.ser == "toms":
            s = self.series_toms(a, b, x, eps)
            return sx("*", ans, sx("+", sx("*", a, s), 1.0))
        # forward from 1/a (w3's accumulation) with the TOMS prefactor times a
        return sx("*", sx("*", ans, a), self.series_w3(a, b, x))

    def alx(self, a, x):
        if self.v.alx == "53":
            return mpf(sx("*", a, LN(x)))
        if self.v.alx == "64":
            return fl64(mpf(a) * mpf(LN(x)))
        return fl64(mpf(a) * bx.W1.ln64(x))

    def w3_pre(self, a, b, x):
        z = self.alx(a, x) - self.lnb_w3(a, b)
        return EXP(fl53(z)) if self.v.z == "z53" else EXPX(fl64(z))

    @staticmethod
    def series_w3(a, b, x):
        s = sx("/", 1.0, a)
        c, n = 1.0, 0.0
        for _ in range(200000):
            n += 1.0
            c = sx("*", sx("*", c, sx("+", sx("-", 0.5, sx("/", b, n)), 0.5)), x)
            s2 = sx("+", s, sx("/", c, sx("+", a, n)))
            if s2 == s:
                break
            s = s2
        return s

    def bpser_w3(self, a, b, x):
        if self.v.eext:
            z = self.alx(a, x) - self.lnb_w3(a, b)
            e = bx.W1.exp_v6_ext(fl64(z) if self.v.z == "z64" else fl53(z))
            return fl53(fl64(e * self.series_w3(a, b, x)))
        return sx("*", self.w3_pre(a, b, x), self.series_w3(a, b, x))


_MODELS: dict = {}


def kernel(a, b, x, y, v: V = V(), cfg=bx.BEST):
    """(w, w1, branch, trace) with bpser variant v. trace = [(fn, a, b, x)] of the series calls that fired."""
    key = (cfg, v)
    m = _MODELS.get(key)
    if m is None:
        m = _MODELS[key] = Model(replace(cfg, w3_z=v.z), v)
    bx._KERNELS[cfg] = m
    m.trace = []
    w, w1, br = bx.kernel(a, b, x, y, cfg)
    bx._KERNELS.pop(cfg, None)
    return w, w1, br, list(m.trace)


def run(points, v: V = V()):
    for _ in range(4):
        out = [kernel(a, b, x, y, v) for a, b, x, y in points]
        if not bx._GL_MISSING:
            return out
        bx.fetch_missing()
    return out


def label(res) -> str:
    """The bratio branch plus the sub-branch of the (single) series call."""
    _, _, br, tr = res
    if not tr:
        return br
    fn, a, b, _ = tr[-1]
    return f"{br}|{sub_branch(fn, a, b)}"


# ---------------------------------------------------------------------------------------------- data
def shape_hold(a: float, b: float) -> bool:
    """Hold-out by SHAPE (md5 odd half of the (a, b) pair), so a fit cannot hide in a few shapes."""
    return int(hashlib.md5(repr((a, b)).encode()).hexdigest(), 16) % 2 == 1


def load_probe(names=PROBES):
    """Rows of this directory's probes: dict(set, blk, a, b, x, y, side, excel, cr, hold) as score_lib.load."""
    rows = []
    cf = HERE / "exact_cache.pkl"
    cache = pickle.loads(cf.read_bytes()) if cf.exists() else {}
    n0 = len(cache)
    for nm in names:
        meta_f, out_f = (
            HERE / f"t2464-r2-bpser-small-{nm}.meta.json",
            HERE / f"t2464-r2-bpser-small-{nm}.out.json",
        )
        if not out_f.exists():
            continue
        out = json.load(open(out_f))["sheets"]["Calc"]
        for i, blk, x, a, b in json.load(open(meta_f)):
            c = out.get(f"A{i}")
            if not c or c.get("t") != "n":
                continue
            rows.append(
                dict(
                    set=nm,
                    blk=blk,
                    a=a,
                    b=b,
                    x=x,
                    y=1.0 - x,
                    side="lower",
                    excel=float(c["v"]),
                    cr=sl._exact_cdf(x, a, b, cache),
                    hold=shape_hold(a, b),
                )
            )
    if len(cache) != n0:
        cf.write_bytes(pickle.dumps(cache))
    return rows


def tally(pairs):
    return sl.tally(pairs)


def fmt(t) -> str:
    n, ex, f, v = t
    return f"{ex}/{f}/{v} of {n}"


def ulps_signed(m: float, e: float) -> float:
    return (m - e) / math.ulp(e) if e else math.inf


def probed_gl() -> dict:
    """Excel's GAMMALN at this probe's shape arguments (case t2464-r2-bpser-small-gl)."""
    f = HERE / "t2464-r2-bpser-small-gl.out.json"
    if not f.exists():
        return {}
    out = json.load(open(f))["sheets"]["Calc"]
    got = {}
    for i, _, v, _, _ in json.load(open(HERE / "t2464-r2-bpser-small-gl.meta.json")):
        c = out.get(f"A{i}")
        if c and c.get("t") == "n":
            got[v] = float(c["v"])
    return got


ENGINE_GL = dict(bx._GL)  # before the override (engine model + r2-bratio's probe)
if bx.USE_PROBED_GL:
    bx._GL.update(probed_gl())


# ---------------------------------------------------------------------------------------------- series variants
def _factor(form, b, n, x):
    """c_n / c_{n-1}, stored x87: "toms" ((0.5 - b/n) + 0.5) x; "1-b/n" (1 - b/n) x; "(n-b)/n" ((n - b)/n) x;
    "(n-b)x/n" ((n - b) x)/n (Cephes pseries)."""
    if form == "toms":
        return sx("*", sx("+", sx("-", 0.5, sx("/", b, n)), 0.5), x)
    if form == "1-b/n":
        return sx("*", sx("-", 1.0, sx("/", b, n)), x)
    if form == "(n-b)/n":
        return sx("*", sx("/", sx("-", n, b), n), x)
    return sx("/", sx("*", sx("-", n, b), x), n)


def series(name: str, a: float, b: float, x: float) -> float:
    """S = 1/a + sum_{n>=1} c_n/(a+n) under several summation orders (all ops stored x87). name = order:form, order
    "fwd1a" (w3: start at fl(1/a), add each term), "fwd0" (sum the terms from 0, add 1/a last), "cephes" (terms
    n >= 2 summed from 0, then + t1, then + 1/a), "pair" (fl(1/a + t1) first, then n >= 2); "mul" (c*x stored before
    the factor: c = (c*x)*f). Stop when the term no longer changes the sum (or underflows)."""
    order, _, form = name.partition(":")
    form = form or "toms"
    n, c = 0.0, 1.0
    terms = []
    for _ in range(200000):
        n += 1.0
        if form == "mul":
            c = sx("*", sx("*", c, x), sx("+", sx("-", 0.5, sx("/", b, n)), 0.5))
        else:
            c = sx("*", c, _factor(form, b, n, x))
        t = sx("/", c, sx("+", a, n))
        terms.append(t)
        if abs(t) < 1e-30 * (1.0 / a) or (len(terms) > 4 and abs(t) < 2.0**-80 / a):
            break
    ia = sx("/", 1.0, a)
    if order == "fwd1a":
        s = ia
        for t in terms:
            s2 = sx("+", s, t)
            if s2 == s:
                break
            s = s2
        return s
    if order == "fwd0":
        s = 0.0
        for t in terms:
            s = sx("+", s, t)
        return sx("+", s, ia)
    if order == "cephes":
        s = 0.0
        for t in terms[1:]:
            s = sx("+", s, t)
        return sx("+", sx("+", s, terms[0]), ia)
    s = sx("+", ia, terms[0])
    for t in terms[1:]:
        s = sx("+", s, t)
    return s


class X87(float):
    """A double whose arithmetic is x87 stored (fl53(fl64(.)) per op): passing X87 values to the double-code TOMS
    pieces (gamln1, algdiv, ...) runs them as Excel's x87 code would. Python prefers a subclass's reflected op, so
    float op X87 also goes through these."""

    def _w(self, op, o, rev=False):
        return X87(sx(op, o, self) if rev else sx(op, self, o))

    def __add__(self, o):
        return self._w("+", o)

    def __radd__(self, o):
        return self._w("+", o, True)

    def __sub__(self, o):
        return self._w("-", o)

    def __rsub__(self, o):
        return self._w("-", o, True)

    def __mul__(self, o):
        return self._w("*", o)

    def __rmul__(self, o):
        return self._w("*", o, True)

    def __truediv__(self, o):
        return self._w("/", o)

    def __rtruediv__(self, o):
        return self._w("/", o, True)

    def __neg__(self):
        return X87(-float(self))


def gamln1_x87(v: float) -> float:
    return float(Model(bx.BEST, V()).gamln1(X87(v)))


def algdiv_x87(a: float, b: float) -> float:
    return float(Model(bx.BEST, V()).algdiv(X87(a), X87(b)))
