"""bratio L140 for a, b > 1 with b0*x0 > 0.7, b0 < 40 and x0 <= 0.7: I = bup(b', a, y, x, n) + bpser(a, b', x), with
b' = frac(b0) in (0, 1] and n = int(b0) (n -= 1 when b' = 0). bpser is the w3 model. bup is TOMS's, Excel-styled:
  bup = P * w,  P = exp(z)/b',  z = b'*ln(y) + a*ln(x) - lnB(b', a)   (brcomp: ln y = log1p(-x) for x <= 0.375)
  w = 1; d = 1; for i in 0 .. n-2: d = d * ((a+b'+i)/(b'+1+i) * y); w = w + d   (x87 stored, TOMS eps 1e-15 stop
  or run to the end)
Variants for ln y, the exponent precision and the stop rule are scored against Excel and CR on these cells.
Usage: python bup.py
"""

import collections
import itertools
import sys

from lib import *
from model import lg_mixed, model

M = mpmath.mpf
from dispatch import branch

rows = [r for r in load(cum=True) if r["a"] != 1.0 and r["b"] != 1.0 and min(r["a"], r["b"]) > 1]
sel = []
for r in rows:
    y = float(sub(1.0, r["x"]))
    br, a0, b0, x0, y0, sw = branch(r["a"], r["b"], r["x"], y)
    if br == "bup+bfrac" and x0 <= 0.7:
        sel.append((r, a0, b0, x0, y0, sw))
print(
    "cells in L140 with x0 <= 0.7:", len(sel), collections.Counter(s[0]["block"] for s in sel).most_common(12)
)
E = np.array([s[0]["excel"] for s in sel])
a0 = np.array([s[1] for s in sel])
b0 = np.array([s[2] for s in sel])
x0 = np.array([s[3] for s in sel])
y0 = np.array([s[4] for s in sel])
sw = np.array([s[5] for s in sel])
nint = np.floor(b0).astype(int)
bp = b0 - nint
z0 = bp == 0
nint[z0] -= 1
bp[z0] = 1.0
cr = np.array([float(exact_cdf(s[0]["x"], s[0]["a"], s[0]["b"])) for s in sel])
save_exact()
# bpser(a0, b', x0) with the w3 model
tail = model(x0, a0, bp, big=8.0)
tail64 = model(x0, a0, bp, big=8.0, zmode="z64")


def bup_sum(y, a, bpr, n, stop):
    A = LD
    w = np.ones_like(y)
    d = np.ones_like(y)
    apb = add(a, bpr)
    ap1 = add(bpr, 1.0)
    active = np.ones(len(y), dtype=bool)
    for i in range(int(n.max()) - 1):
        act = active & (i < n - 1)
        if not act.any():
            break
        r = mul(div(add(apb, float(i)), add(ap1, float(i))), y)
        dn = mul(d, r)
        wn = add(w, dn)
        d = np.where(act, dn, d)
        w = np.where(act, wn, w)
        if stop == "eps":
            active &= ~(dn <= 1e-15 * wn)
    return w


# exponent pieces
lnx = xl("LN", x0)
alx = mul(a0, lnx)
lny_opts = {
    "LN(y)": xl("LN", y0),
    "log1p(-x) cr": np.array([float(mpmath.log1p(-M(v))) for v in x0]),
    "brcomp: log1p if x<=.375": np.where(
        x0 <= 0.375, np.array([float(mpmath.log1p(-M(v))) for v in x0]), xl("LN", y0)
    ),
}
la, lb, lab = lg_mixed(bp), lg_mixed(a0), lg_mixed(add(a0, bp))
Lb = []
for i in range(len(sel)):
    if max(a0[i], bp[i]) > 8:
        Lb.append(fl64(mpmath.log(mpmath.beta(M(float(bp[i])), M(float(a0[i]))))))
    else:
        Lb.append(fl64(la[i] + fl64(lb[i] - lab[i])))
res = {}
ws = {st: bup_sum(y0, a0, bp, nint, st) for st in ("eps", "all")}
for (lyn, lny), zm, st in itertools.product(lny_opts.items(), ("z53", "z64"), ("eps", "all")):
    bly = mul(bp, lny)
    zz = [M(p) + M(q) - L for p, q, L in zip(alx, bly, Lb)]
    if zm == "z53":
        e = xl("EXP", np.array([fl53(v) for v in zz]))
    else:
        e = np.array([fl53(mpmath.exp(fl64(v))) for v in zz])
    P = div(e, bp)
    bupv = mul(P, ws[st])
    for tn, t in (("tail z53", tail), ("tail z64", tail64)):
        w = add(bupv, t)
        out = np.where(sw, add(sub(0.5, w), 0.5), w)
        res[(lyn, zm, st, tn)] = out
blocks = collections.defaultdict(list)
for i, s in enumerate(sel):
    blocks[s[0]["block"]].append(i)
print(f"CR: {fmt(score(cr, E))}")
ranked = sorted(res.items(), key=lambda kv: -score(kv[1], E)[0])
for k, v in ranked[:8]:
    print(f"  {score(v, E)[0]:5d}/{score(v, E)[1]:5d}/{score(v, E)[2]:5d}  {k}")
best = ranked[0][1]
for bk, idx in sorted(blocks.items(), key=lambda t: -len(t[1])):
    idx = np.array(idx)
    print(
        f"   {bk:12s} n {len(idx):5d}  best {score(best[idx], E[idx])[0]:5d}  CR {score(cr[idx], E[idx])[0]:5d}"
    )


def tie_c(vals, idx):
    """w2's tie response: (mean err at fl(1-x) rounded up - rounded down) / (2 |rel rounding|) (units)."""
    xs = np.array([sel[i][0]["x"] for i in idx])
    d = np.array([float((M(float(sub(1.0, v))) - (1 - M(v))) / (1 - M(v))) / U for v in xs])
    ex = [exact_cdf(sel[i][0]["x"], sel[i][0]["a"], sel[i][0]["b"]) for i in idx]
    e = rel_units(vals[idx], ex)
    up, dn = e[d > 1e-9], e[d < -1e-9]
    mag = np.abs(d[np.abs(d) > 1e-9]).mean()
    return (up.mean() - dn.mean()) / (2 * mag) if len(up) and len(dn) else float("nan")


for bk in ("xruns-1:0", "xruns-1:1", "xruns-1:2", "xruns-1:5", "xruns-2:3"):
    idx = np.array(blocks[bk])
    print(f"   tie response {bk}: Excel {tie_c(E, idx):+.2f}  model {tie_c(best, idx):+.2f}  CR {tie_c(cr, idx):+.2f}")
