# early-only ridge LS to Excel's function + ladder rows holding grid_a's late stage values at 2..2.13; env LAM, STSIG, IT
import numpy as np, pickle, sys, mpmath, math, os
from k23lib import P2, Q2
from bits import nextd

mpmath.mp.dps = 40
co = pickle.load(open("gcorr6.pkl", "rb"))
LAM = float(os.environ.get("LAM", "1"))
STSIG = float(os.environ.get("STSIG", "0.3"))
which = [("p", i) for i in range(5)] + [("q", i) for i in range(3)]
p = list(P2)
q = list(Q2)
nodes = [mpmath.mpf(2.5) - mpmath.mpf(0.5) * mpmath.cos(mpmath.pi * (i + 0.5) / 200) for i in range(200)]


def R(p, q, x):
    n = mpmath.mpf(0)
    d = mpmath.mpf(0)
    for c in p:
        n = n * x + mpmath.mpf(c)
    for c in q:
        d = d * x + mpmath.mpf(c)
    return n, d


tgt = [
    R(P2, Q2, x)[0] / R(P2, Q2, x)[1]
    + mpmath.mpf(float(np.polynomial.legendre.legval(float((x - 2.5) * 2), co)))
    for x in nodes
]
X0 = (2.0, 2.01, 2.03, 2.06, 2.1, 2.13)
REF = {}
for x0 in X0:
    for w, c, js in (("p", list(P2), range(5, 9)), ("q", list(Q2), range(4, 8))):
        sv = 0.0
        for j in range(len(c)):
            sv = sv * x0 + c[j] if j else c[0]
            if j in js:
                REF[(w, j, x0)] = sv
for it in range(int(os.environ.get("IT", "4"))):
    A = []
    b = []
    LADR = []
    LADB = []
    for x, t in zip(nodes, tgt):
        n, d = R(p, q, x)
        S = n / d
        A.append(
            [
                float(x ** (len(p) - 1 - i) / n * math.ulp(abs(p[i])))
                if w == "p"
                else float(-(x ** (len(q) - 1 - i)) / d * math.ulp(abs(q[i])))
                for w, i in which
            ]
        )
        b.append(float((t - S) / S))
    for x0 in X0:
        for w, c, js in (("p", p, range(5, 9)), ("q", q, range(4, 8))):
            sv = 0.0
            for j in range(len(c)):
                sv = sv * x0 + c[j] if j else c[0]
                if j in js:
                    u = math.ulp(abs(sv))
                    LADR.append(
                        [
                            (math.ulp(abs(c[i])) * x0 ** (j - i) / u / STSIG) if (ww == w and i <= j) else 0.0
                            for ww, i in which
                        ]
                    )
                    LADB.append((REF[(w, j, x0)] - sv) / u / STSIG)
    A = np.array(A)
    b = np.array(b)
    sig = 2e-19
    M = np.vstack([A / sig, np.array(LADR), LAM * np.eye(A.shape[1])])
    bb = np.concatenate([b / sig, np.array(LADB), np.zeros(A.shape[1])])
    sol = np.linalg.lstsq(M, bb, rcond=None)[0]
    for (w, i), s in zip(which, sol):
        arr = p if w == "p" else q
        arr[i] = nextd(arr[i], int(round(s)))
    print(it, "moves", [int(round(s)) for s in sol], flush=True)
pickle.dump((p, q), open(sys.argv[1], "wb"))
