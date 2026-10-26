"""In-sample a1/a2: forward selection of rounding features for Excel's density error (units 2^-53 relative).
Each feature is S * (stored - ideal) of one candidate intermediate, where ideal is computed from the previously
stored pieces, and S is the sensitivity of ln f to it (units).  usage: s11_fsel.py [blk-substring]"""
import sys
from collections import defaultdict

import mpmath
import numpy as np

import tdlib as TD

mp = mpmath.mpf
U = 2.0**-53
L, st = TD.LDB, TD.st
flt = sys.argv[1] if len(sys.argv) > 1 else "t4.3"
rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if flt in r["blk"]]
by = defaultdict(list)
for r in rows:
    by[(r["n"], r["blk"])].append(r)


def feats(t, n):
    X, N = L(t), L(n)
    h = (n + 1) / 2
    H = mp(h)
    f = {}
    tt = st(X * X)
    f["tt"] = -H / (n + mp(t) ** 2) * (mp(tt) - mp(t) ** 2)
    q = st(L(tt) / N)
    f["q"] = -H / (1 + mp(q)) * (mp(q) - mp(tt) / n)
    w = st(L(1) + L(q))
    f["w"] = -H / mp(w) * (mp(w) - 1 - mp(q))
    u = st(N + L(tt))
    f["u"] = -H / mp(u) * (mp(u) - n - mp(tt))
    y = st(N / L(u))
    f["y"] = H / mp(y) * (mp(y) - n / mp(u))
    z = st(L(tt) / L(u))
    f["z"] = -H / (1 - mp(z)) * (mp(z) - mp(tt) / mp(u))
    L1 = TD.LN(w)
    f["LNw"] = -H * (mp(L1) - mpmath.log(mp(w)))
    Lu = TD.LN(u)
    f["LNu"] = -H * (mp(Lu) - mpmath.log(mp(u)))
    Ly = TD.LN(y)
    f["LNy"] = H * (mp(Ly) - mpmath.log(mp(y)))
    lp = TD.log1p_xl(q)
    f["l1p_q"] = -H * (mp(lp) - mpmath.log1p(mp(q)))
    lz = TD.log1p_xl(-z)
    f["l1p_-z"] = H * (mp(lz) - mpmath.log1p(-mp(z)))
    for nm, Lv, sg in (("w", L1, -1), ("u", Lu, -1), ("y", Ly, 1)):
        p = st(L(h) * L(Lv))
        f[f"h*LN{nm}"] = sg * (mp(p) - H * mp(Lv))
        p2 = st(L(n / 2) * L(Lv))
        f[f"n/2*LN{nm}"] = sg * (mp(p2) - mp(n) / 2 * mp(Lv))
        # E = fl(c + sg*p) with the exact constant c (|E| rounding)
        c = TD.exact_logpdf(t, n) - sg * H * mp(Lv)  # constant that makes E exact before rounding
        E = mp(c) + sg * mp(p)
        f[f"E(h*LN{nm})"] = mp(TD.st(TD.ld_of(E))) - E
    sw = st(np.sqrt(L(w)))
    f["sqrt w"] = -(mp(sw) - mpmath.sqrt(mp(w))) / mp(sw)
    return {k: float(v) / U for k, v in f.items()}


for key in sorted(by):
    rr = by[key]
    n = key[0]
    F = [feats(r["t"], n) for r in rr]
    names = list(F[0])
    Xm = np.array([[d[k] for k in names] for d in F])
    Y = np.array([float((r["excel"] - mpmath.exp(TD.exact_logpdf(r["t"], n))) / mpmath.exp(TD.exact_logpdf(r["t"], n)))
                  / U for r in rr])
    chosen = []
    line = f"n={n:<3g} {key[1]:16s} sd {Y.std():5.2f}"
    for _ in range(4):
        best = None
        for j, nm in enumerate(names):
            if j in chosen or Xm[:, j].std() < 1e-9:
                continue
            A = np.column_stack([Xm[:, chosen + [j]], np.ones(len(Y))])
            c, *_ = np.linalg.lstsq(A, Y, rcond=None)
            s = (Y - A @ c).std()
            if best is None or s < best[0]:
                best = (s, j, c)
        chosen.append(best[1])
        line += f" | +{names[best[1]]}({best[2][-2]:+.2f}) -> {best[0]:.2f}"
    print(line, flush=True)
