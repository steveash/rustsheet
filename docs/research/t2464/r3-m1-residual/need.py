"""Needed exponent offset per row: the interval of absolute offsets d (units of 2^-58) such that Excel's EXP (V6)
of E + d equals Excel's density, E = M1's exact exponent (for a = 1: n*l - lnB). Writes need_<set>.tsv
(x, a, b, E, t = E*log2(e), dlo, dhi).  Usage: need.py log1p|probe1|old|<own> [step] [D]"""
import sys

import mpmath

import m1lib as M
from lscan import rows_of

U = M.mpf(2) ** -58


def exponent(r):
    """M1's exponent as an mpf (exact sum of the stored pieces; b = 1 stored)."""
    a, b = r["a"], r["b"]
    if "l" in r:
        tx, ty = 0.0, M.sx("*", b - 1.0, r["l"])
    else:
        tx, ty = M.pieces(r["x"], a, b)
    L = M.mpf(M.lnb_base(a, b))
    if b == 1.0:
        return M.mpf(M.fl53(M.mpf(tx) - L))
    return M.mpf(tx) + M.mpf(ty) - L


def need(r, D=48):
    E = exponent(r)
    ok = [d for d in range(-D, D + 1) if M.W1.exp_v6(E + d * U) == r["excel"]]
    return E, (min(ok), max(ok)) if ok else (None, None)


if __name__ == "__main__":
    name = sys.argv[1]
    step = int(sys.argv[2]) if len(sys.argv) > 2 else 1
    D = int(sys.argv[3]) if len(sys.argv) > 3 else 48
    rows = rows_of(name)[::step]
    with open(M.HERE / f"need_{name}.tsv", "w") as f:
        for r in rows:
            E, (lo, hi) = need(r, D)
            t = E / mpmath.ln(2)
            f.write(f"{r['x']!r}\t{r['a']!r}\t{r['b']!r}\t{float(E)!r}\t{mpmath.nstr(t, 20)}\t{lo}\t{hi}\n")
