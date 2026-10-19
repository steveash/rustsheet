# common setup for grid3/ridge/climb: early-stage anchor-phase table, null-space followers, coherence
import numpy as np, pickle, sys, math, os
from nullb import rows, WHICH

p, q, T = pickle.load(open(sys.argv[1], "rb"))
T = [
    e for e in T if e["idx"] in [int(v) for v in os.environ.get("KEEP", "4,5,12,13").split(",")]
]  # stages validated by the control
A = rows(p, q)
ip1 = WHICH.index(("p", 1))
iq0 = WHICH.index(("q", 0))
rest = [k for k in range(len(WHICH)) if k not in (ip1, iq0)]
reg = np.vstack([A[:, rest], 1e-6 * np.eye(len(rest))])


def follow(col):  # other early coefs following a unit move of col, by function+ladder LS
    sol = np.linalg.lstsq(reg, np.concatenate([-A[:, col], np.zeros(len(rest))]), rcond=None)[0]
    w = np.zeros(len(WHICH))
    w[col] = 1
    w[rest] = sol
    return w


w1 = follow(ip1)
w2 = follow(iq0)
n = len(p) - 1
idx = [i if w == "p" else n + 1 + i for w, i in WHICH]
ulps = np.array([math.ulp(abs(p[i])) if w == "p" else math.ulp(abs(q[i])) for w, i in WHICH])
Gf = np.array([[e["a"][k] * ulps[j] for j, k in enumerate(idx)] for e in T])
ph = np.array([e["ph"] for e in T])
keys = {}
for r, e in enumerate(T):
    keys.setdefault((e["idx"], round(e["x0"], 6)), []).append(r)
Gg = np.array([Gf[rs].mean(0) for rs in keys.values()])
gph = np.array([np.angle(np.exp(2j * np.pi * ph[rs]).mean()) / (2 * np.pi) for rs in keys.values()])
gw = np.array([len(rs) for rs in keys.values()], float)
gw /= gw.sum()
gg1 = Gg @ w1
gg2 = Gg @ w2


def coh(dc):
    return float((np.cos(2 * np.pi * (Gg @ dc - gph)) * gw).sum())


def refine(dc, steps=(-3, -2, -1, 1, 2, 3)):  # integer polish of the non-wide coefs by coherence
    cur = coh(dc)
    for rnd in range(4):
        imp = False
        for k in range(len(WHICH)):
            if k in (ip1, iq0):
                continue
            for s in steps:
                d2 = dc.copy()
                d2[k] += s
                v = coh(d2)
                if v > cur:
                    cur, dc, imp = v, d2, True
        if not imp:
            break
    return dc, cur
