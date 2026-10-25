"""a7: matched regression of the residual shape offset (needed minus exact Loader at Excel's k, n, nk) on the
rounding errors of many candidate stored pieces of the constant, each signed as it would enter E.

Groups of candidates:
  loader : LN(n+1) (+1); LN(k), LN(nk) (-1/2), LN(n) (+1/2); the stored LF0 partial sums (-1/2 each)
  klog   : k LN(fl(k/n)) and nk LN(fl(nk/n)) (+1 each): ratio, LN and product roundings
  gl     : Excel GAMMALN errors at a, b, a+b (TOMS/GAMMALN B form: -1, -1, +1) and at k+1, nk+1, n+1 (R formula)
  stf    : R's stirlerr formula pieces for each of n, k, nk (sign +1 for n, -1 for k, nk): GAMMALN(v+1) error,
           fl((v+.5) LN v) rounding, and its LN(v) rounding (zero where v is a half-integer <= 15 or v > 15)
Prints the full fit, then each group alone. Usage: a7_creg.py [sweep|diverse|both]
"""

from __future__ import annotations

import csv
import sys

import mpmath
import numpy as np

import a3_const as A3
import r3lib as R

mpf = mpmath.mpf
U = mpf(2) ** -53
D = R.L.D


def LN(t):
    return mpf(R.L.LN(float(t)))


def rd(v):
    """Stored x87 rounding error of v (fl53(fl64(v)) - v)."""
    return mpf(D(v)) - v


def klog_feats(k, n):
    r = mpf(D(k / n))
    l = LN(r)
    t = mpf(D(k * l))
    return [k * (r - k / n) / r, k * (l - mpmath.log(r)), t - k * l]


def stf_feats(v):
    v = mpf(v)
    if v > 15 or 2 * v == int(2 * v):
        return [0.0, 0.0, 0.0]
    g = mpf(R.GL(float(D(v + 1)))) - mpmath.loggamma(mpf(D(v + 1)))
    lv = LN(v)
    p = mpf(D(mpf(D(v + 0.5)) * lv))
    return [g, -(p - mpf(D(v + 0.5)) * lv), -(v + 0.5) * (lv - mpmath.log(v))]


def feats(a, b):
    k, n, nk = (mpf(v) for v in R.shape_pieces(a, b))
    f, names = [], []

    def add(name, val):
        names.append(name)
        f.append(float(val / U))

    add("L5", LN(n + 1) - mpmath.log(n + 1))
    add("LNk", -(LN(k) - mpmath.log(k)) / 2)
    add("LNnk", -(LN(nk) - mpmath.log(nk)) / 2)
    add("LNn", (LN(n) - mpmath.log(n)) / 2)
    s1 = mpf(A3.H.M_LN_2PI) + LN(k)
    s2 = mpf(D(s1)) + LN(nk)
    s3 = mpf(D(s2)) - LN(n)
    add("lf.s1", -rd(s1) / 2)
    add("lf.s2", -rd(s2) / 2)
    add("lf.s3", -rd(s3) / 2)
    for nm, v in zip(("kl.r", "kl.ln", "kl.mul"), klog_feats(k, n)):
        add(nm, v)
    for nm, v in zip(("nkl.r", "nkl.ln", "nkl.mul"), klog_feats(nk, n)):
        add(nm, v)
    for nm, v, s in (("GLa", a, -1), ("GLb", b, -1), ("GLab", float(D(mpf(a) + mpf(b))), 1)):
        add(nm, s * (mpf(R.GL(v)) - mpmath.loggamma(mpf(v))))
    for tag, v, s in (("n", n, 1), ("k", k, -1), ("nk", nk, -1)):
        for nm, val in zip(("gl", "mul", "ln"), stf_feats(v)):
            add(f"st{tag}.{nm}", s * val)
    return names, f


GROUPS = {
    "loader": ("L5", "LNk", "LNnk", "LNn", "lf.s1", "lf.s2", "lf.s3"),
    "klog": ("kl.r", "kl.ln", "kl.mul", "nkl.r", "nkl.ln", "nkl.mul"),
    "gl": ("GLa", "GLb", "GLab"),
    "stf": tuple(f"st{t}.{m}" for t in ("n", "k", "nk") for m in ("gl", "mul", "ln")),
}


def main():
    which = sys.argv[1] if len(sys.argv) > 1 else "both"
    files = ["sweep", "diverse"] if which == "both" else [which]
    shapes = []
    for w in files:
        for r in csv.DictReader(open(R.HERE / f"off_{w}_R.tsv"), delimiter="\t"):
            shapes.append((float(r["a"]), float(r["b"]), float(r["offset_units"])))
    vs = []
    for a, b, _ in shapes:
        vs += [a, b, float(D(mpf(a) + mpf(b)))]
        vs += [float(D(mpf(v) + 1)) for v in R.shape_pieces(a, b)]
    R.gl_prefetch(vs)
    Y, F = [], []
    names = None
    for a, b, o in shapes:
        p = float((A3.cand(a, b, ("exact", "exact"), "ex", "m", "ex") - R.exact_const(a, b)) / U)
        names, f = feats(a, b)
        Y.append(o - p)
        F.append(f + [1.0])
    Y, F = np.array(Y), np.array(F)
    c, *_ = np.linalg.lstsq(F, Y, rcond=None)
    print(f"{which}: {len(Y)} shapes, residual sd {Y.std():.2f} -> {(Y - F @ c).std():.2f} (all features)")
    print("   " + ", ".join(f"{n_}={v:+.2f}" for n_, v in zip(names + ["1"], c)))
    for g, cols in GROUPS.items():
        idx = [names.index(cn) for cn in cols] + [len(names)]
        cg, *_ = np.linalg.lstsq(F[:, idx], Y, rcond=None)
        print(f"   group {g:7s} sd -> {(Y - F[:, idx] @ cg).std():.2f}: " + ", ".join(f"{cn}={v:+.2f}" for cn, v in zip(cols, cg)))


if __name__ == "__main__":
    main()
