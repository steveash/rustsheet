"""Implied ln B per shape. For every unswapped w_bpser cell, with w3's series s (forward from 1/a, x87 stored) and
alx = fl(a LN x), invert Excel's I = fl(E * s), E = EXP(z), z = alx - L (z extended), into an interval of L. Intersect
the intervals of a shape's cells: the implied L of that shape (or a conflict: then the series or the composition is
wrong for some cell). Then compare candidate ln B forms with the implied interval, per shape and per sub-branch.

EXP is treated as correctly rounded at the extended argument (Excel's EXP V6 error is ~2^-58 relative for |z| < 40),
so intervals are exact to about 1/30 ulp of I; a candidate counts as "in" when inside the interval widened by 2% of
its width. Offsets are printed in units of 2^-53 (relative to the result, i.e. absolute units in L).
usage: lscan.py [sets=shapes] [--list]"""

from __future__ import annotations

import math
import sys
from collections import Counter, defaultdict
from dataclasses import replace

import mpmath

import lib
from lib import mpf, sx

U = 2.0**-53


def e_interval(e: float, s: float):
    """Doubles E with fl(E*s) == e (x87 stored product): (lo, hi) or None."""
    g = e / s
    cand = [g]
    for _ in range(8):
        cand.append(math.nextafter(cand[-1], math.inf))
    lo = g
    for _ in range(8):
        lo = math.nextafter(lo, -math.inf)
        cand.append(lo)
    ok = sorted(E for E in cand if sx("*", E, s) == e)
    return (ok[0], ok[-1]) if ok else None


def l_interval(a, b, x, e, ser=None):
    s = lib.Model.series_w3(a, b, x) if ser is None else lib.series(ser, a, b, x)
    ei = e_interval(e, s)
    if ei is None:
        return None
    lo, hi = ei
    zlo = mpmath.log(mpf(lo) - mpf(math.ulp(lo)) / 2)
    zhi = mpmath.log(mpf(hi) + mpf(math.ulp(hi)) / 2)
    alx = mpf(sx("*", a, lib.LN(x)))
    return alx - zhi, alx - zlo


def candidates(a, b):
    """Candidate ln B values (mpf) for bpser's (a, b)."""
    out = {}
    with mpmath.workdps(40):
        out["acc"] = mpmath.log(mpmath.beta(mpf(a), mpf(b)))
    for lg in ("toms", "gl", "acc", "toms53"):
        m = lib.Model(replace(lib.bx.BEST, w3_lg=lg if lg != "toms53" else "toms"), lib.V(lg=lg))
        out[f"w3:{lg}"] = mpf(m.lnb_w3(a, b))
        out[f"w3:{lg}:min"] = mpf(lib.Model(replace(m.c, w3_order="min"), lib.V(lg=lg)).lnb_w3(a, b))
    k = lib.Model(lib.bx.BEST, lib.V())
    out["toms_betaln"] = mpf(k.betaln(a, b))  # TOMS betaln with Excel GAMMALN/LN (double)
    gl = lib.GAMMALN
    lo, hi = min(a, b), max(a, b)
    out["gl53"] = mpf(sx("+", gl(lo), sx("-", gl(hi), gl(a + b))))  # all stored
    out["gl53:a+(b-ab)"] = mpf(sx("+", gl(a), sx("-", gl(b), gl(a + b))))
    out["gl53:(a+b)-ab"] = mpf(sx("-", sx("+", gl(a), gl(b)), gl(a + b)))
    out["gl64:(a+b)-ab"] = lib.fl64(lib.fl64(mpf(gl(a)) + gl(b)) - gl(a + b))
    tk = lib.bx.Kernel(replace(lib.bx.BEST, gamln="toms"))  # TOMS gamln with Excel LN (double code)

    def tg(v):  # TOMS gamln below 1 (gamln1(v) - LN v; v > 0.8: gamln1(v - 1)), Excel GAMMALN from 1
        return tk.gamln(v) if v < 1.0 else gl(v)

    out["mix53"] = mpf(sx("+", tg(lo), sx("-", gl(hi), gl(a + b))))
    out["mix53:g1-ln"] = mpf(sx("+", sx("-", k.gamln1(lo), lib.LN(lo)), sx("-", gl(hi), gl(a + b))))
    if hi >= 8.0:
        out["mix:algdiv"] = mpf(sx("+", tg(lo), k.algdiv(lo, hi)))
        out["mix:algdiv64"] = lib.fl64(mpf(tg(lo)) + k.algdiv(lo, hi))
        out["gl+algdiv"] = mpf(sx("+", gl(lo), k.algdiv(lo, hi)))
    return out


def main():
    kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
    sets = tuple(kw.get("sets", "shapes").split(","))
    old = [s for s in sets if s not in lib.PROBES]
    rows = (lib.sl.load(tuple(old)) if old else []) + lib.load_probe(
        tuple(s for s in sets if s in lib.PROBES)
    )
    pts = [(r["a"], r["b"], r["x"], r["y"]) for r in rows]
    res = lib.run(pts, lib.V())
    shapes = defaultdict(list)
    for r, q in zip(rows, res):
        if q[2] != "w_bpser" or r["side"] != "lower":
            continue
        fn, a, b, x = q[3][-1]
        shapes[(a, b)].append((x, r["excel"]))
    tally = Counter()
    for (a, b), cells in sorted(shapes.items()):
        sub = lib.sub_branch("bpser", a, b)
        lo, hi, conflict, nint = -mpmath.inf, mpmath.inf, False, 0
        for x, e in cells:
            iv = l_interval(a, b, x, e)
            if iv is None:
                continue
            nint += 1
            nlo, nhi = max(lo, iv[0]), min(hi, iv[1])
            if nlo > nhi:
                conflict = True
                continue
            lo, hi = nlo, nhi
        tally[(sub, "shapes")] += 1
        if conflict or nint == 0:
            tally[(sub, "conflict")] += 1
            continue
        c = candidates(a, b)
        w = (hi - lo) * 0.02
        hits = [k for k, v in c.items() if lo - w <= v <= hi + w]
        for k in hits:
            tally[(sub, k)] += 1
        if not hits:
            tally[(sub, "no candidate")] += 1
        if "--list" in sys.argv:
            acc = c["acc"]
            print(
                f"{sub:13s} a={a:<8g} b={b:<8g} n={nint} L-acc in [{float((lo - acc) / U):+8.2f}, "
                f"{float((hi - acc) / U):+8.2f}] "
                + " ".join(f"{k}={float((v - acc) / U):+.2f}" for k, v in c.items() if k != "acc")
                + f"  hits={hits}"
            )
    for sub in sorted({k[0] for k in tally}):
        print(sub, {k[1]: v for k, v in sorted(tally.items()) if k[0] == sub})


if __name__ == "__main__":
    main()
