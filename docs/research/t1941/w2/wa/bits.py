import numpy as np, sys

sys.path.insert(0, ".")
from ysearch import Scorer, nextd
from k23lib import P2, Q2

W = "../w/"
# plateau window per stage: (log2 finest seen grid, log2 coarsest S grid) in ulp(c) units; index into p/q, highest degree first
WIN = {
    ("p", 5): (1, 2),
    ("p", 6): (7, 8),
    ("p", 7): (3, 4),
    ("p", 8): (6, 8),
    ("p", 9): (10, 13),
    ("q", 3): (-2, 1),
    ("q", 4): (0, 1),
    ("q", 5): (1, 2),
    ("q", 6): (4, 5),
    ("q", 7): (7, 8),
    ("q", 8): (12, 14),
}
import ctypes

_L = ctypes.CDLL("./libpoly.so")
_DP = ctypes.POINTER(ctypes.c_double)


def sim(p, q, X):
    y = np.zeros_like(X)
    pa = np.ascontiguousarray(p, float)
    qa = np.ascontiguousarray(q, float)
    _L.gbase(
        pa.ctypes.data_as(_DP),
        len(pa),
        qa.ctypes.data_as(_DP),
        len(qa),
        X.ctypes.data_as(_DP),
        len(X),
        y.ctypes.data_as(_DP),
    )
    return y


class Stair:
    """chi2 of per-bin mean residual (data - sim) in ulps of y; bins of width bw"""

    def __init__(s, X, Y, bw=0.002):
        s.X = np.ascontiguousarray(X)
        s.Y = Y
        s.u = np.spacing(np.abs(Y))
        s.b = np.floor((X - 2) / bw).astype(int)
        s.b -= s.b.min()
        s.nb = np.bincount(s.b)
        s.ok = s.nb > 0

    def means(s, p, q):
        r = (s.Y - sim(p, q, s.X)) / s.u
        return np.bincount(s.b, r)[s.ok] / s.nb[s.ok], np.bincount(s.b, r * r)[s.ok] / s.nb[s.ok]

    def __call__(s, p, q):
        m, _ = s.means(p, q)
        return -float(np.sum(m * m * s.nb[s.ok]))


def loadall(synth=None):
    X = np.concatenate([np.load(W + f + "X.npy") for f in ["all", "lad", "e6", "e8"]])
    Y = np.concatenate([np.load(W + f + "Y.npy") for f in ["all", "lad", "e6", "e8"]])
    o = np.argsort(X, kind="stable")
    X = X[o]
    Y = Y[o]
    u, i = np.unique(X, return_index=True)
    X = X[i]
    Y = Y[i]
    m = (X > 2.005) & (X <= 3) & np.isfinite(Y)
    X = np.ascontiguousarray(X[m])
    Y = Y[m]  # x<=2.005 excluded (misparse row)
    if synth is not None:
        Y = sim(synth[0], synth[1], X)
    return X, Y


def patterns(p, q, st, win=WIN, ext=1):
    """per stage: candidate offsets t (multiples of the finest grid, +-2^(hi+ext)) and the bin-mean change each produces alone"""
    m0, _ = st.means(p, q)
    pats = {}
    for (w, i), (lo, hi) in win.items():
        step = 2 ** max(lo, 0)
        R = 2 ** (hi + ext)
        ts = list(range(-R, R + 1, step))
        G = []
        for t in ts:
            pp = list(p)
            qq = list(q)
            arr = pp if w == "p" else qq
            arr[i] = nextd(arr[i], t)
            G.append(st.means(pp, qq)[0] - m0)
        pats[(w, i)] = (ts, np.array(G))
    return m0, pats


def polish(r0, pats, rounds=3):
    """additive-pattern solve: pairwise descent + basin hopping (fix each stage at each value, descend the rest); pats pre-weighted"""
    keys = list(pats)
    mid = {k: pats[k][0].index(0) for k in keys}

    def descend(ch, fixed):
        G = {k: pats[k][1] for k in keys}
        while True:
            tot = r0 + sum(G[k][ch[k]] for k in keys)
            cur = float(tot @ tot)
            imp = False
            free = [k for k in keys if k != fixed]
            for a in range(len(free)):
                for b in range(a, len(free)):
                    ka, kb = free[a], free[b]
                    base = tot - G[ka][ch[ka]] - (G[kb][ch[kb]] if b != a else 0)
                    if a == b:
                        c = np.sum((base + G[ka]) ** 2, axis=1)
                        i = int(np.argmin(c))
                        j = None
                        cv = c[i]
                    else:
                        Ga = G[ka]
                        Gb = G[kb]
                        c = np.sum((base[None, None, :] + Ga[:, None, :] + Gb[None, :, :]) ** 2, axis=2)
                        i, j = np.unravel_index(int(np.argmin(c)), c.shape)
                        cv = c[i, j]
                    same = (int(i) == ch[ka]) and (j is None or int(j) == ch[kb])
                    if (not same) and cv < cur * (
                        1 - 1e-9
                    ) - 1e-12:  # 'same' guard: the expanded cost otherwise loops forever
                        ch[ka] = int(i)
                        if j is not None:
                            ch[kb] = int(j)
                        tot = r0 + sum(G[k][ch[k]] for k in keys)
                        cur = float(tot @ tot)
                        imp = True
            if not imp:
                return cur, ch

    ch = dict(mid)
    cur, ch = descend(ch, None)
    for r in range(rounds):
        better = False
        for k in keys:
            for v in range(len(pats[k][0])):
                if v == ch[k]:
                    continue
                c2 = dict(ch)
                c2[k] = v
                cv, c2 = descend(c2, k)
                if cv < cur * (1 - 1e-9) - 1e-12:
                    cur, ch = cv, c2
                    better = True
        if not better:
            break
    return cur, {k: pats[k][0][v] for k, v in ch.items()}
