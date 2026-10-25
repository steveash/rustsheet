"""m1: full models of Excel's density for a, b > 1 and their scores (exact / within 4 ulp / within 15 digits).

Model family (all on Excel's stored k = a-1, n = fl(a+b)-2, nk = fl(n-k)):
  X  = r2-density-large's x87 bd0 part (log form t = fl(k LN(fl(k/M))) + (M - k), cut 0.2, series extended)
  C  = L5 + Cs - lf/2 with
       L5 : 'xl' Excel LN(n+1) | 'm' exact ln(n+1)
       Cs : exact stirlerr (n) - (k) - (nk)
       lf : 'LF0x' fl(fl(fl(ln2pi + LN k) + LN nk) - LN n) | 'LF0e' the same sums extended | 'LF0s' extended
            then stored once | 'm' exact
  bd0st: each bd0 value rounded to a double before X is summed (M4*)
  E  = C + X kept extended (EXP_V6 on the extended value)  | 'chainR': r2's LDe chain E = fl(L5 + fl(lc - lf/2))
Usage: m1_model.py DATASET [MODEL...]   (model CR = the correctly rounded value, r2 only)   DATASET in sweep, diverse, grid, r2, hold (hold only once frozen)
"""

from __future__ import annotations

import sys

import mpmath

import a3_const as A3
import r3lib as R

mpf = mpmath.mpf
D = R.L.D

MODELS = {
    "M3": dict(l5="xl", lf="LF0e", chain="ext"),
    "M3x": dict(l5="xl", lf="LF0x", chain="ext"),
    "M3m": dict(l5="xl", lf="m", chain="ext"),
    "Mexact": dict(l5="m", lf="m", chain="ext"),
    "LDe-r2": dict(l5="xl", lf="LF0x", chain="chainR"),
    "M4": dict(l5="xl", lf="LF0e", chain="ext", bd0st=True),
    "M4s": dict(l5="xl", lf="LF0s", chain="ext", bd0st=True),
    "M4x": dict(l5="xl", lf="LF0x", chain="ext", bd0st=True),
}
_CC: dict = {}


def const(a, b, o):
    key = (a, b, o["l5"], o["lf"])
    if key not in _CC:
        k, n, nk = (mpf(v) for v in R.shape_pieces(a, b))
        L5 = mpf(R.L.LN(float(D(n + 1)))) if o["l5"] == "xl" else mpmath.log(n + 1)
        ex = ("exact", "exact")
        Cs = A3.stirlerr(n, ex) - A3.stirlerr(k, ex) - A3.stirlerr(nk, ex)
        if o["lf"] == "m":
            lf = A3.lf_val(k, n, nk, "m", "m")
        elif o["lf"] == "LF0s":  # extended sum, stored once (an assigned variable)
            lf = mpf(D(A3.lf_val(k, n, nk, "LF0", "e")))
        else:
            lf = A3.lf_val(k, n, nk, "LF0", "e" if o["lf"] == "LF0e" else "x")
        _CC[key] = (L5, Cs, lf)
    return _CC[key]


def density(x, a, b, o):
    L5, Cs, lf = const(a, b, o)
    X, b1, b2 = R.LD.xpart(x, a, b, **R.BEST_X)
    if o.get("bd0st"):  # each bd0 returned as a double
        X = R.L.R64(-mpf(D(b1)) - mpf(D(b2)))
    if o["chain"] == "chainR":
        lc = mpf(D(Cs + X))
        E = mpf(D(L5 + mpf(D(lc - lf / 2))))
    else:
        E = L5 + Cs - lf / 2 + X
    return R.L.exp_v6(E)


def dataset(name):
    if name == "r2":
        return [
            dict(blk=r["blk"], x=r["x"], a=r["a"], b=r["b"], excel=r["excel"], src=r["src"], cr=float(r["exact"]))
            for r in R.r2_rows()
        ]
    rows = R.load(name)
    if name == "hold":  # correctly rounded reference for the hold-out (the in-sample probes do not need it)
        for r in rows:
            A, B, X = mpf(r["a"]), mpf(r["b"]), mpf(r["x"])
            r["cr"] = float(X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B))
    return rows


def main():
    ds = sys.argv[1]
    rows = dataset(ds)
    names = sys.argv[2:] or list(MODELS)
    for nm in names:
        o = MODELS.get(nm)
        sub: dict = {}
        for r in rows:
            p = r["cr"] if nm == "CR" else density(r["x"], r["a"], r["b"], o)
            key = ("all",)
            if ds == "hold":
                key = ("all", "sweeps" if r["blk"].startswith("sw") else "shapes")
            if ds == "r2":
                key = ("all", "r2-p1" if r["src"].startswith("probe_") else "round-1",
                       "min>=8" if min(r["a"], r["b"]) >= 8 else "1<min<8")
            for kk in key:
                sub.setdefault(kk, []).append((p, r["excel"]))
        print(f"{ds} {nm}: " + " | ".join(f"{kk} {R.L.fmt(R.L.tally(v))}" for kk, v in sub.items()))
        sys.stdout.flush()


if __name__ == "__main__":
    main()
