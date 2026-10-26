"""r2-tdist: Excel-candidate overrides of r2-bratio's TOMS 708 port, for the Student t branches (b = 1/2).

XCfg extends bratio_excel.Cfg with:
  alnrel  "toms" (TOMS (3,3) rational), "xl" (r2-log1p's Excel log1p: atanh series in r = a/(a+2), x87 stored,
          |a| <= 0.375; LN(fl(1+a)) beyond)
  erfc    "toms" (TOMS erf/erfc1 in grat1 at a = 1/2) or "xl" (Excel's ERF / ERFC.PRECISE, engine bit-exact models via
          the python bindings)
  bg_lnx  how bgrat forms ln x (its only dependence on x and y): "alnrel" (TOMS: LN(x) if y > 0.375 else
          alnrel(-y)), "ln" (LN(x) always), "cut<v>" (LN(x) if y > v else alnrel(-y)), "y1x" (alnrel(-fl(1-x)))
  bg      "toms" (the port's bgrat) or "ideal": at b = 1/2, bgrat's increment is RN(F(a, lnx) Q_xl / Q_exact), the
          exact I at ln x = lnx times Excel's ERFC(fl(sqrt(fl(-nu lnx)))) over the exact erfc (hx.py), added to w;
          "ld": the implementable form, TOMS bgrat at b = 1/2 in x87 extended with those stored pieces (bgrat_half_ld)
`kernel(a, b, x, y, xcfg)` is r2-bratio's kernel with this Kernel subclass registered for the config.
"""

from __future__ import annotations

import math
import pickle
import sys
from dataclasses import dataclass

import mpmath

import tlib

sys.path.insert(0, str(tlib.T2464 / "r2-log1p"))
import excel_log1p  # noqa: E402

import englib  # noqa: E402

bx = tlib.bx
sx = bx.sx


@dataclass(frozen=True)
class XCfg(bx.Cfg):
    alnrel: str = "toms"
    erfc: str = "toms"
    bg_lnx: str = "alnrel"
    bg: str = "toms"


class XKernel(bx.Kernel):
    def alnrel(self, a: float) -> float:
        if self.c.alnrel == "xl" and abs(a) <= 0.375:
            return excel_log1p.log1p_series(a) if a != 0 else 0.0
        return super().alnrel(a)

    def erf(self, x: float) -> float:
        if self.c.erfc == "xl":
            return englib.eng_lazy("ERF.PRECISE", x, super().erf(x))
        return super().erf(x)

    def erfc1(self, ind: int, x: float) -> float:
        if self.c.erfc == "xl" and ind == 0:
            return englib.eng_lazy("ERFC.PRECISE", x, super().erfc1(0, x))
        return super().erfc1(ind, x)


    def bgrat(self, a: float, b: float, x: float, y: float, w: float, eps: float) -> float:
        # TOMS bgrat reads y only to choose lnx = LN(x) (y > 0.375) or alnrel(-y); steer that choice by y.
        m = self.c.bg_lnx
        if m == "ln" or (m.startswith("cut") and y > float(m[3:])):
            y = 0.5
        elif m.startswith("cut"):
            y = min(y, 0.375)
        elif m == "y1x":
            y = 1.0 - x
        if self.c.bg in ("ideal", "ld") and b == 0.5:
            lnx = self.log(x) if y > 0.375 else self.alnrel(-y)
            if self.c.bg == "ld":
                return bgrat_half_ld(a, lnx, w)
            return float(mpmath.mpf(w) + bgrat_half_ideal(a, lnx))
        return super().bgrat(a, b, x, y, w, eps)


_FF = tlib.HERE / "hx_cache.pkl"
FC: dict = pickle.loads(_FF.read_bytes()) if _FF.exists() else {}


def save_cache():
    _FF.write_bytes(pickle.dumps(FC))


def F(a: float, lnx: float):
    """Exact I_y(a, 1/2) at ln y = lnx (mpmath, 50 digits)."""
    k = (a, lnx)
    if k not in FC:
        with mpmath.workdps(50):  # the direct lower tail: 1 - I(1/2, a, 1 - y) cancels in deep tails
            L = mpmath.mpf(lnx)
            try:
                FC[k] = mpmath.betainc(a, 0.5, 0, mpmath.exp(L), regularized=True)
            except ValueError:  # no convergence for y next to 1 at huge a, where the complement is safe
                FC[k] = 1 - mpmath.betainc(0.5, a, 0, -mpmath.expm1(L), regularized=True)
    return FC[k]


def Q_xl(s: float) -> float:
    """Excel's Q(1/2, s^2) as TOMS grat1 forms it: erfc(s) for s^2 >= 0.25, else 0.5 + (0.5 - erf(s))."""
    if s * s >= 0.25:
        return englib.eng_lazy("ERFC.PRECISE", s, math.erfc(s))
    return 0.5 + (0.5 - englib.eng_lazy("ERF.PRECISE", s, math.erf(s)))


def bgrat_half_ideal(a: float, lnx: float):
    """hx.py's identity for bgrat at b = 1/2 (an mpmath value; the caller rounds)."""
    nu = a - 0.25
    zz = -nu * lnx
    with mpmath.workdps(50):
        qx = mpmath.erfc(mpmath.sqrt(-mpmath.mpf(nu) * mpmath.mpf(lnx)))
        return F(a, lnx) * Q_xl(math.sqrt(zz)) / qx


def kernel(a, b, x, y, cfg: XCfg):
    if cfg not in bx._KERNELS:
        bx._KERNELS[cfg] = XKernel(cfg)
    if x == 0.0:
        return 0.0, 1.0, "x=0"
    return bx.kernel(a, b, x, y, cfg)


def run(points, cfg: XCfg):
    """kernel() over [(a, b, x, y)], fetching missing engine/GAMMALN values and re-evaluating."""
    for _ in range(4):
        out = [kernel(*p, cfg) for p in points]
        if not englib.MISSING and not bx._GL_MISSING:
            return out
        englib.fetch_missing()
        if bx._GL_MISSING:
            bx.fetch_missing()
    return out


LD = bx.np.longdouble


def _algdiv_ld(a, b):
    """TOMS algdiv ln(Gamma(b)/Gamma(a+b)), b >= 8, every operation in x87 extended (numpy longdouble)."""
    a, b = LD(a), LD(b)
    if a > b:
        h = b / a
        c, x, d = 1 / (h + 1), h / (h + 1), a + (b - LD(0.5))
    else:
        h = a / b
        c, x, d = h / (h + 1), 1 / (h + 1), b + (a - LD(0.5))
    x2 = x * x
    s, acc = [], LD(1)
    for _ in range(5):
        acc = x + x2 * acc + 1
        s.append(acc)
    t = (1 / b) ** 2
    co = (0.0833333333333333, -0.00277777777760991, 7.9365066682539e-4, -5.9520293135187e-4,
          8.37308034031215e-4, -0.00165322962780713)
    w = LD(co[5]) * s[4]
    for k in range(4, 0, -1):
        w = w * t + LD(co[k]) * s[k - 1]
    w = (w * t + LD(co[0])) * (c / b)
    u = d * bx.np.log1p(a / b)
    v = a * (bx.np.log(b) - 1)
    return w - v - u if u > v else w - u - v


def bgrat_half_ld(a: float, lnx: float, w: float = 0.0) -> float:
    """TOMS bgrat at b = 1/2 in x87 extended, with the stored pieces Excel shows: lnx (a double from Excel's LN or
    log1p), z = fl(-nu lnx), Q = Excel's ERFC(fl(sqrt(z))) (grat1's a = 1/2 path); result rounded once.
    TOMS eps = 15 * 1e-15 stop rule kept."""
    b = LD(0.5)
    nu = LD(a) - LD(0.25)
    zd = float(-nu * LD(lnx))  # stored double: it feeds SQRT and ERFC
    L = LD(lnx)
    z = LD(zd)
    r = bx.np.exp(b * bx.np.log(z) - z) / bx.np.sqrt(LD(bx.np.pi)) * 2 * b  # b z^b e^-z / Gamma(b+1), Gamma(1.5)=sqrt(pi)/2
    E = bx.np.exp(-(_algdiv_ld(0.5, a) + b * bx.np.log(nu)))
    u = r * E
    q_r = LD(Q_xl(math.sqrt(zd))) / r
    v = LD(0.25) / (nu * nu)
    t2 = LD(0.25) * L * L
    l = LD(w) / u if w else LD(0)
    j, s = q_r, q_r
    t, cn, n2 = LD(1), LD(1), LD(0)
    cs, ds = [], []
    for n in range(1, 31):
        bp2n = b + n2
        j = (bp2n * (bp2n + 1) * j + (z + bp2n + 1) * t) * v
        n2 += 2
        t *= t2
        cn /= n2 * (n2 + 1)
        cs.append(cn)
        acc, coef = LD(0), b - n
        for i in range(1, n):
            acc += coef * cs[i - 1] * ds[n - 1 - i]
            coef += b
        ds.append((b - 1) * cn + acc / n)
        dj = ds[-1] * j
        s += dj
        if s <= 0 or abs(dj) <= LD(15e-15) * (s + l):
            break
    return float(LD(w) + u * s)
