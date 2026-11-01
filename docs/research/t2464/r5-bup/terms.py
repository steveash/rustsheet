"""T-3568 r5-bup: score W forms on the terms probes (Excel's density at every bup shape c + j).

Per point: Excel's L140 cell E and D (t3568-r5-bup-<half>, columns A and B) and P_j = BETA.DIST(y0, c+j, a0, FALSE)
(t3568-r5-bup-terms-<half>). Models, each RN(W + D):
  REC   the recurrence: x87(P_0 fl(xy)/c) times the exact sum (score.py's best fixed form)
  LOOPx sum_j of one-term bups, each x87(P_j fl(xy)/(c+j)) stored; the sum in doubles from j = 0 up (x87: extended)
  LOOPd the same with each term fl(fl(P_j fl(xy))/(c+j))
  LOOPrev the double sum from the top shape down
and, as diagnostics, the sd of eW - eP_top (units of 2^-53): if the top density's error is Excel's W error, it is
small. usage: terms.py tune|hold"""
from __future__ import annotations

import json
import statistics
import sys
from collections import defaultdict

import mpmath
import numpy as np

import score as s
from common import CASES, sl

LD = np.longdouble
M = mpmath.mpf
U = M(2) ** -53
COLS = [chr(ord("A") + i) for i in range(26)] + [f"A{chr(ord('A') + i)}" for i in range(13)]

half = sys.argv[1]
base = {r["k"]: r for r in s.load(half)}
out = json.load(open(CASES / f"t3568-r5-bup-terms-{half}.oracle.json"))["sheets"]["Calc"]
g = defaultdict(lambda: defaultdict(list))
diag = defaultdict(list)
rep = [0, 0]
for k, src, a0, b0, x0, y0, n, c in json.load(open(s.HERE / f"t3568-r5-bup-terms-{half}.meta.json")):
    r = base[src]
    ps = [s.num(out.get(f"{COLS[j]}{k}")) for j in range(n)]
    if None in ps:
        continue
    rep[0] += 1
    rep[1] += ps[0] == r["P"]
    xy = x0 * y0
    tx = [float(LD(p) * LD(xy) / LD(c + j)) for j, p in enumerate(ps)]
    td = [p * xy / (c + j) for j, p in enumerate(ps)]
    up = 0.0
    for t in tx:
        up += t
    upd = 0.0
    for t in td:
        upd += t
    upl = LD(0)
    for t in tx:
        upl += LD(t)
    dn = 0.0
    for t in reversed(tx):
        dn += t
    rec = s.model(r, "x87", "ex", "x87")
    nb = "n<=9" if n <= 9 else "n>=10"
    vals = {"REC (x87 ret * exact S)": rec, "LOOPx up, double sum": up + r["D"], "LOOPd up, double sum": upd + r["D"],
            "LOOPx up, x87 sum": float(LD(upl) + LD(r["D"])), "LOOPx down, double sum": dn + r["D"],
            "exact sum of x87 terms": float(mpmath.fsum([M(t) for t in tx]) + r["D"]),
            "exact sum of exact P_j xy/(c+j)": float(mpmath.fsum([M(p) * xy / (c + j) for j, p in enumerate(ps)]) + r["D"])}
    for m, v in vals.items():
        g[nb][m].append((v, r["excel"]))
        g["all"][m].append((v, r["excel"]))
    # diagnostics: Excel's W against the exact recurrence on the top density
    yv, xv = M(y0), M(x0)
    se = s.bup_sum(c, a0, y0, x0, n, "ex")
    pt0 = yv ** (c - 1) * xv ** (a0 - 1) / mpmath.beta(c, a0)
    wt = pt0 * xy / c * se
    ew = float(((M(r["excel"]) - r["D"]) / wt - 1) / U)
    ept = [float((M(p) / (yv ** (c + j - 1) * xv ** (a0 - 1) / mpmath.beta(c + j, a0)) - 1) / U) for j, p in enumerate(ps)]
    wts = [M(t) for t in td]
    tot = sum(wts)
    emix = float(sum(w * e for w, e in zip(wts, ept)) / tot)
    diag[nb].append((ew - ept[0], ew - ept[-1], ew - emix))
print(f"terms probe {half}: {rep[0]} points; P_0 equals probe column C on {rep[1]}\n")
print("| model | " + " | ".join(sorted(g)) + " |")
print("|---|" + "---|" * len(g))
for m in g["all"]:
    line = f"| {m} |"
    for key in sorted(g):
        nn, e, f, w = sl.tally(g[key][m])
        line += f" {e}/{f}/{w} ({nn}) |"
    print(line)
print("\n| n | sd eW-eP_0 | sd eW-eP_top | sd eW - term-weighted eP_j |")
print("|---|---|---|---|")
for key, v in sorted(diag.items()):
    print(f"| {key} | " + " | ".join(f"{statistics.pstdev([t[i] for t in v]):.1f}" for i in range(3)) + " |")
