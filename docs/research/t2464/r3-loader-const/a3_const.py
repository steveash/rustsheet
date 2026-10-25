"""a3: score candidate shape constants C = LN(n+1) + Cs - lf/2 against the needed offsets of a2 (off_<which>_R.tsv).

Needed offset o (units 2^-53) = Excel's constant minus the exact one at R's stored k, n, nk. A candidate predicts
p = C_cand - Cex. Reported: rms(o - p), the share of shapes with |o - p| <= 1 and <= 2, and the sd of o - p, per
group: k and n-k both half-integers (table), one non-half-integer <= 15 (formula), both > 15 (series).
Usage: a3_const.py sweep|diverse [--formula]   (--formula: the stirlerr-formula variants lgamma x arithmetic)
"""

from __future__ import annotations

import csv
import math
import statistics as st
import sys

import mpmath

import r3lib as R

mpf = mpmath.mpf
U = mpf(2) ** -53
H = R.M2.H  # w6 h2_loader: SF table, S0..S4, M_LN_2PI, M_LN_SQRT_2PI
D, R64 = R.L.D, R.L.R64


def rnd(v, ar):
    return mpf(D(v)) if ar == "x" else (R64(v) if ar == "e" else mpf(v))


def st_exact(v):
    v = mpf(v)
    return mpmath.loggamma(v + 1) - (v + 0.5) * mpmath.log(v) + v - mpmath.log(mpmath.sqrt(2 * mpmath.pi))


def series(v, ar, cuts=(15, 35, 80, 500)):
    r = lambda t: rnd(t, ar)  # noqa: E731
    S0, S1, S2, S3, S4 = (mpf(s) for s in (H.S0, H.S1, H.S2, H.S3, H.S4))
    v = mpf(v)
    nn = r(v * v)
    if v > cuts[3]:
        return r(r(S0 - r(S1 / nn)) / v)
    if v > cuts[2]:
        return r(r(S0 - r(r(S1 - r(S2 / nn)) / nn)) / v)
    if v > cuts[1]:
        return r(r(S0 - r(r(S1 - r(r(S2 - r(S3 / nn)) / nn)) / nn)) / v)
    return r(r(S0 - r(r(S1 - r(r(S2 - r(r(S3 - r(S4 / nn)) / nn)) / nn)) / nn)) / v)


def formula(v, lg, ar):
    """lgamma(v+1) - (v+.5) ln v + v - ln sqrt(2 pi); lg 'GL' Excel GAMMALN(v+1), 'GLv' GAMMALN(v)+LN(v), 'ex' exact
    lgamma rounded to a double; ln is Excel's LN."""
    lnm = "xl"
    if ar.endswith("L"):  # LN kept extended (ln64) instead of Excel's stored LN
        ar, lnm = ar[:-1], "e"
    r = lambda t: rnd(t, ar)  # noqa: E731
    v = mpf(v)
    if lg == "GL":
        g = mpf(R.GL(float(D(v + 1))))
    elif lg == "GLv":
        g = r(mpf(R.GL(float(v))) + mpf(R.L.LN(float(v))))
    elif lg == "XT":  # lgamma held to 64 bits
        g = R64(mpmath.loggamma(v + 1))
    else:
        g = mpf(float(mpmath.loggamma(v + 1)))
    lv = R.L.ln64(v) if lnm == "e" else mpf(R.L.LN(float(v)))
    t = r(r(v + 0.5) * lv)
    return r(r(r(g - t) + v) - mpf(H.M_LN_SQRT_2PI))


def stirlerr(v, kind):
    """kind = (small, big): small in {'exact','GL-x','GL-e','GLv-x','ex-x'} for non-half-integers <= 15 (half-integers
    use R's table); big in {'exact','ser-x','ser-e','ser-m'} above 15. 'tab-all-exact': exact everywhere."""
    small, big = kind
    v = mpf(v)
    if v <= 15:
        if (2 * v) == int(2 * v) and small != "exact":
            return mpf(H.SF[int(2 * v)])
        if small == "exact":
            return st_exact(v)
        lg, ar = small.split("-")
        return formula(v, lg, ar)
    if big == "exact":
        return st_exact(v)
    return series(v, big.split("-")[1])


def lf_val(k, n, nk, form, ar):
    r = lambda t: rnd(t, ar)  # noqa: E731
    LN = lambda t: mpf(R.L.LN(float(t)))  # noqa: E731
    if form == "LF0":
        return r(r(r(mpf(H.M_LN_2PI) + LN(k)) + LN(nk)) - LN(n))
    if form == "LF1":
        return r(r(mpf(H.M_LN_2PI) + LN(k)) + mpf(R.L.log1p_xl(float(D(-k / n)))))
    return mpmath.log(2 * mpmath.pi) + mpmath.log(k) + mpmath.log(nk) - mpmath.log(n)


def cand(a, b, kind, lf="LF0", lfar="x", l5="xl"):
    k, n, nk = (mpf(v) for v in R.shape_pieces(a, b))
    Cs = stirlerr(n, kind) - stirlerr(k, kind) - stirlerr(nk, kind)
    L5 = mpf(R.L.LN(float(D(n + 1)))) if l5 == "xl" else mpmath.log(n + 1)
    return L5 + Cs - lf_val(k, n, nk, lf, lfar) / 2


def group(a, b):
    k, n, nk = (float(v) for v in R.shape_pieces(a, b))
    small = [v for v in (k, n, nk) if v <= 15 and 2 * v != int(2 * v)]
    if small:
        return "formula"
    if max(k, nk) <= 15 or min(k, nk) <= 15:
        return "table+ser"
    return "series"


CANDS_F = {
    f"table + {lg}-{ar} formula + ser-x": ((f"{lg}-{ar}", "ser-x"), "LF0", "x", "xl")
    for lg in ("GL", "ex", "XT")
    for ar in ("x", "e", "eL", "m", "mL")
}
CANDS = {
    "exact (all exact pieces)": (("exact", "exact"), "ex", "m", "ex"),
    "exact stirlerr, LF0 x87, LN5": (("exact", "exact"), "LF0", "x", "xl"),
    "exact stirlerr, LF0 extended": (("exact", "exact"), "LF0", "e", "xl"),
    "exact stirlerr, LF1 x87": (("exact", "exact"), "LF1", "x", "xl"),
    "table + ser-x, small exact": (("exact", "ser-x"), "LF0", "x", "xl"),
    "table + GL formula x87 + ser-x": (("GL-x", "ser-x"), "LF0", "x", "xl"),
    "table + GL formula ext + ser-e": (("GL-e", "ser-e"), "LF0", "x", "xl"),
    "table + GLv formula x87 + ser-x": (("GLv-x", "ser-x"), "LF0", "x", "xl"),
    "table + exact-lgamma formula x87": (("ex-x", "ser-x"), "LF0", "x", "xl"),
}


def main():
    which = sys.argv[1]
    rows = list(csv.DictReader(open(R.HERE / f"off_{which}_R.tsv"), delimiter="\t"))
    shapes = [(float(r["a"]), float(r["b"]), float(r["offset_units"]), int(r["cells"])) for r in rows]
    R.gl_prefetch(
        [float(D(mpf(v) + 1)) for a, b, _, _ in shapes for v in R.shape_pieces(a, b)]
        + [float(v) for a, b, _, _ in shapes for v in R.shape_pieces(a, b)]
    )
    groups = sorted({group(a, b) for a, b, _, _ in shapes})
    print(f"{which}: {len(shapes)} shapes; groups " + ", ".join(f"{g} {sum(group(a, b) == g for a, b, _, _ in shapes)}" for g in groups))
    cands = CANDS_F if "--formula" in sys.argv else CANDS
    for name, (kind, lf, lfar, l5) in cands.items():
        res = {g: [] for g in groups}
        for a, b, o, n in shapes:
            p = float((cand(a, b, kind, lf, lfar, l5) - R.exact_const(a, b)) / U)
            res[group(a, b)].append(o - p)
        line = []
        for g in groups:
            d = res[g]
            rms = math.sqrt(sum(v * v for v in d) / len(d))
            line.append(f"{g}: rms {rms:6.2f} mean {st.mean(d):+6.2f} |d|<=1 {sum(abs(v) <= 1 for v in d):3d} <=2 {sum(abs(v) <= 2 for v in d):3d}/{len(d)}")
        print(f"  {name:36s} " + " | ".join(line))


if __name__ == "__main__":
    main()
