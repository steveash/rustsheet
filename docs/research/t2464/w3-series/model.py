"""Current best model of Excel's direct-branch I_x(a, b) for min(a, b) < 1 (w3-series, T-2464).

  lg(v)  = ln Gamma(v): accurate, kept in extended precision (64-bit) for v < 1; Excel's GAMMALN (a double) for v >= 1
  L      = fl64(lg(a) + fl64(lg(b) - lg(a+b)))           (ln B in x87 registers, never stored)
  alx    = fl53(fl64(a * LN(x)))                          (Excel's LN, the product stored to a double)
  z      = fl53(alx - L)                                  (the EXP argument, a double)
  s      = fl(1/a) + sum_{n>=1} c_n/(a+n),  c_n = c_{n-1} * (((0.5 - b/n) + 0.5) * x)  (forward accumulation,
           x87 stored ops, stop when s no longer changes)
  I      = fl53(fl64(EXP(z) * s))                         (Excel's EXP)
The rec/tf/mode/start options select the series variants that were compared (series.accum2).
"""

from __future__ import annotations

from lib import *
from series import accum2

M = mpmath.mpf


OVERRIDE = {}


def lg_mixed(v, thresh=1.0, src="exact64"):
    """ln Gamma per element: 64-bit accurate below thresh, Excel GAMMALN otherwise (list of mpf).
    OVERRIDE maps a double argument to a forced ln Gamma value (experiments)."""
    g = xl("GAMMALN", v)
    out = []
    for vi, gi in zip(np.asarray(v, dtype=float), g):
        if float(vi) in OVERRIDE:
            out.append(M(OVERRIDE[float(vi)]))
        else:
            out.append(fl64(mpmath.loggamma(M(vi))) if vi < thresh else M(gi))
    return out


def exponent(x, a, b, thresh=1.0, grouping="a+(b-ab)", inner64=True, alx_mode="LN53", big=None):
    """alx_mode: 'LN53' fl53(a*LN(x)) with Excel's LN double; 'ln64' fl53(a*fl64(ln x)) (ln kept extended).
    big: if set (e.g. 8), shapes with max(a,b) > big use the accurate 64-bit lnGamma for all three terms."""
    apb = add(a, b)
    la, lb, lab = lg_mixed(a, thresh), lg_mixed(b, thresh), lg_mixed(apb, thresh)
    bigL = {}
    if big is not None:
        for i in range(len(la)):
            if max(a[i], b[i]) > big:
                # accurate ln B as one extended value (TOMS-style algdiv avoids the cancellation)
                bigL[i] = fl64(mpmath.log(mpmath.beta(M(float(a[i])), M(float(b[i])))))
    if alx_mode == "LN53":
        alx = mul(a, xl("LN", x))
    else:
        alx = np.array([fl53(M(float(ai)) * fl64(mpmath.log(M(float(xi))))) for ai, xi in zip(a, x)])
    rnd = fl64 if inner64 else (lambda v: M(fl53(fl64(v))))
    z = []
    for i in range(len(alx)):
        if i in bigL:
            L = bigL[i]
        elif grouping == "a+(b-ab)":
            L = fl64(la[i] + rnd(lb[i] - lab[i]))
        elif grouping == "(a+b)-ab":
            L = fl64(rnd(la[i] + lb[i]) - lab[i])
        else:
            L = fl64(lb[i] + rnd(la[i] - lab[i]))
        z.append(M(alx[i]) - L)
    return z


def exp_of(z, zmode="z53"):
    """z53: Excel's EXP of fl53(z); z64: exp of the extended argument fl64(z), rounded once to a double (V6 with an
    extended input is correctly rounded except near ties; approximated by correct rounding here)."""
    if zmode == "z53":
        return xl("EXP", np.array([fl53(v) for v in z]))
    return np.array([fl53(mpmath.exp(fl64(v))) for v in z])


def model(x, a, b, mode="x87", rec="R4", tf="T0", start="1/a", stop="unchanged", zmode="z53", **kw):
    x, a, b = (np.asarray(v, dtype=float) for v in (x, a, b))
    s, nt = accum2(x, a, b, mode=mode, rec=rec, tf=tf, start=start, stop=stop)
    z = exponent(x, a, b, **kw)
    return mul(exp_of(z, zmode), s)
