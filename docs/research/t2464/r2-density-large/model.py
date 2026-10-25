"""Full (no free constant) candidate models of Excel's density for a, b > 1: Loader's dbinom_raw chain with Excel pieces.

    k = a-1, n = (a+b)-2, nk = n-k                         (x87 stored)
    Cs = stirlerr(n) - stirlerr(k) - stirlerr(nk)          (w6 Arith: table at half-integers <= 15, else Excel
                                                            GAMMALN(n+1) - (n+.5) LN(n) + n - ln sqrt(2 pi); series > 15)
    X  = -bd0(k, n x) - bd0(nk, n fl(1-x))                 (loader.xpart, knobs)
    lc = fl(Cs + X)
    lf = LF0: LN(2pi) + LN(k) + LN(nk) - LN(n)  |  LF1: ln 2pi + LN(k) + log1p(-k/n)
    E  = chain(...)  and density = EXP_V6(E)
Options are a dict; `predict(rows, opts)` returns the predictions.
"""

from __future__ import annotations

import sys

import mpmath

import dlib as L
import loader as LD

sys.path.append(str(L.T / "w6-literature"))
import h2_loader as H  # noqa: E402

# route w6's GAMMALN calls to this directory's cache (never write w1-density's cache)
H.xl = lambda name, xs: [L.GL(v) for v in xs] if name == "GAMMALN" else L.excelmath.excel(name, list(xs))

M_LN_2PI = mpmath.mpf(H.M_LN_2PI)
BEST_X = dict(ar="x", lnmode="xl", addar="e", cut=0.2, sar="e", xar="e")


def lf_value(k, n, form, ar="x"):
    r = lambda v: L.rnd(v, ar)  # noqa: E731
    k, n = mpmath.mpf(k), mpmath.mpf(n)
    if form == "LF0":
        return r(
            r(r(M_LN_2PI + mpmath.mpf(L.LN(float(k)))) + mpmath.mpf(L.LN(float(n - k))))
            - mpmath.mpf(L.LN(float(n)))
        )
    if form == "LF1":
        return r(r(M_LN_2PI + mpmath.mpf(L.LN(float(k)))) + mpmath.mpf(L.log1p_xl(float(r(-k / n)))))
    return mpmath.log(2 * mpmath.pi) + mpmath.log(k) + mpmath.log(n - k) - mpmath.log(n)  # exact


_SHAPE: dict = {}


def shape_consts(a, b, opts):
    key = (a, b, opts.get("stir", "D"), opts.get("lf", "LF0"), opts.get("lfar", "x"))
    if key not in _SHAPE:
        A = H.Arith(opts.get("stir", "D"))
        k = L.D(mpmath.mpf(a) - 1)
        n = L.D(mpmath.mpf(L.D(mpmath.mpf(a) + b)) - 2)
        nk = L.D(mpmath.mpf(n) - k)
        if opts.get("stir") == "exact":
            st = lambda v: (
                mpmath.loggamma(mpmath.mpf(v) + 1)
                - (mpmath.mpf(v) + 0.5) * mpmath.log(v)
                + v
                - mpmath.log(mpmath.sqrt(2 * mpmath.pi))
            )  # noqa: E731,E501
        else:
            st = A.stirlerr
        Cs = st(n) - st(k) - st(nk)
        lf = lf_value(k, n, opts.get("lf", "LF0"), opts.get("lfar", "x"))
        L5 = mpmath.mpf(L.LN(float(L.D(mpmath.mpf(L.D(mpmath.mpf(a) + b)) - 1))))
        _SHAPE[key] = (k, n, nk, Cs, lf, L5)
    return _SHAPE[key]


def density(x, a, b, opts):
    k, n, nk, Cs, lf, L5 = shape_consts(a, b, opts)
    X = LD.xpart(x, a, b, **opts.get("xk", BEST_X))[0]
    lc = L.rnd(Cs + X, opts.get("lc", "x"))
    h = lf / 2
    ch = opts.get("chain", "R")
    if ch == "R":  # E = fl(L5 + fl(lc - lf/2))
        E = L.rnd(L5 + L.rnd(lc - h, opts.get("g", "x")), "x")
    elif ch == "K":  # E = fl(fl(L5 - lf/2) + lc)
        E = L.rnd(mpmath.mpf(L.D(L5 - h)) + lc, "x")
    else:  # extended sum, stored
        E = L.rnd(L5 + lc - h, "x")
    return L.exp_v6(E)


def predict(rows, opts, idx=None):
    idx = range(len(rows)) if idx is None else idx
    return {i: density(rows[i]["x"], rows[i]["a"], rows[i]["b"], opts) for i in idx}
