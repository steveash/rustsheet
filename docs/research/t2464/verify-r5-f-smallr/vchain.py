"""verify-r5-f-smallr: own test of the claimed one-chain structure S1 = fl(C + H), S2 = fl(S1 + P), F = fl(EXP(E)/x).
Per cell, the E interval that yields Excel's double exactly is [ln(lo x), ln(hi x)], lo/hi = Excel's value -+ half an
ulp (EXP is accurate far below that).  For C on an exact 1/4-unit lattice around K_exact (+-SPAN units):
  pure  E = S2 exactly (one free parameter C)                      -> cells whose interval contains S2
  +D    E = S2 + D, D a free constant outside the roundings, |D| <= DMAX units (two free parameters per run)
The best count over the lattice (and D) is reported per run.  H = -h L(r) extended (own series), P = +fl(a LN x) |
-fl(b LN x) with Excel's LN (shared excelmath harness).  Usage: vchain.py CASE_ID [CASE_ID ...]"""
from __future__ import annotations

import collections
import math
import sys
from pathlib import Path

import mpmath
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
import excelmath as XM  # noqa: E402

import vcheck as VC  # noqa: E402
import vparse as V  # noqa: E402
import vscore as S  # noqa: E402

LD = np.longdouble
SPAN, STEP, DMAX = 320, 0.25, 64.0
TOL = None  # None: the exact value interval; else +-TOL units around the cell centre (the worker's style of score)
U53 = mpmath.mpf(2) ** 53


def best_stab(lo, hi, dmax):
    """Max number of intervals [lo_i, hi_i] sharing a point D with |D| <= dmax, and that D."""
    ev = []
    for a, b in zip(lo, hi):
        a, b = max(a, -dmax), min(b, dmax)
        if a <= b:
            ev += [(a, 0), (b, 1)]
    ev.sort()
    k = best = 0
    at = None
    for v, t in ev:
        k += 1 if t == 0 else -1
        if t == 0 and k > best:
            best, at = k, v
    return best, at


def run(m, n, side, rows):
    a, b, h = m / 2, n / 2, (m + n) / 2
    lnx = XM.excel("LN", [x for x, _ in rows])
    K = VC.K_exact(m, n, side)
    Kh = float(K)
    klo = float((K - Kh) * U53)
    cells = []
    for (x, e), lx in zip(rows, lnx):
        _, r = S.side_r(x, m, n)
        H = -LD(h) * LD(VC.log1p_ser(r))
        P = float(LD(a) * LD(lx)) if side == 1 else -float(LD(b) * LD(lx))
        u = math.ulp(e) / 2
        X = mpmath.mpf(x)
        lo_, hi_ = mpmath.log((mpmath.mpf(e) - u) * X), mpmath.log((mpmath.mpf(e) + u) * X)
        if TOL is not None:
            mid = (lo_ + hi_) / 2
            lo_, hi_ = mid - TOL / U53, mid + TOL / U53
        cells.append((H, P, lo_, hi_))
    res = {"pure": (-1, None), "+D": (-1, None)}
    for c in np.arange(-SPAN, SPAN + STEP, STEP):
        off = float(np.round((klo + c) / STEP) * STEP)
        C = LD(Kh) + LD(off) * LD(2.0) ** -53
        lo, hi = [], []
        for H, P, l, u in cells:
            S2 = float(LD(float(C + H)) + LD(P))
            lo.append(float((l - S2) * U53))
            hi.append(float((u - S2) * U53))
        p = sum(1 for a_, b_ in zip(lo, hi) if a_ <= 0 <= b_)
        if p > res["pure"][0]:
            res["pure"] = (p, off)
        d = best_stab(lo, hi, DMAX)
        if d[0] > res["+D"][0]:
            res["+D"] = (d[0], (off, d[1]))
    return res, len(cells)


if __name__ == "__main__":
    want = sys.argv[1]  # 'eq' (m = n runs only) or 'all'
    if sys.argv[2].startswith("tol="):
        TOL = float(sys.argv.pop(2)[4:])
    g = collections.defaultdict(list)
    for cid in sys.argv[2:]:
        for cell, x, m, n, e in V.rows(cid)[0]:
            side, r = S.side_r(x, m, n)
            if r <= 0.375 and (want == "all" or m == n):
                g[(cid.split("-")[-1], int(m), int(n), side, round(r, 2))].append((x, e))
    tot = collections.Counter()
    for k, rows in sorted(g.items()):
        res, nn = run(k[1], k[2], k[3], rows)
        tot["n"] += nn
        tot["pure"] += res["pure"][0]
        tot["+D"] += res["+D"][0]
        print(k, f"n {nn}  pure C: {res['pure'][0]} (C-K = {res['pure'][1]})  C + outside D: {res['+D'][0]} "
                 f"(C-K, D = {res['+D'][1][0]}, {res['+D'][1][1] and round(res['+D'][1][1], 2)})")
        sys.stdout.flush()
    print("total", dict(tot))
