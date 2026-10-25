"""Diagnostic: per block, Excel's density error vs exact (units 2^-53 rel): mean, sd; CR score; and the best constant
c such that EXP_V6(E_exact + c) (E in extended) or RN(exp(E)*(1+c)) fits, as a ceiling for 'accurate exponent then EXP'."""
import statistics as st

import mpmath

import dlib as L

rows = L.load()
for blk, idx in sorted(L.blocks(rows).items()):
    r0 = rows[idx[0]]
    e = [L.err_units(rows[i]["excel"], rows[i]["exact"]) for i in idx]
    best = None
    for c16 in range(-12, 13):
        c = mpmath.mpf(c16) / 4 * mpmath.mpf(2) ** -53
        n = 0
        for i in idx[:300]:
            E = mpmath.log(rows[i]["exact"]) + c
            n += L.exp_v6(L.R64(E)) == rows[i]["excel"]
        if best is None or n > best[1]:
            best = (c16 / 4, n)
    print(f"{blk:14s} ({r0['a']:g},{r0['b']:g}) n={len(idx):5d} mean {st.mean(e):+6.2f} sd {st.pstdev(e):5.2f}"
          f"  CR {L.fmt(L.tally((float(rows[i]['exact']), rows[i]['excel']) for i in idx))}"
          f"  bestEXPV6(E+c) c={best[0]:+.2f} -> {best[1]}/{min(300, len(idx))}", flush=True)
