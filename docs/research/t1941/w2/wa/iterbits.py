# plateau bit-reader. args: in|grid_a out legendre_deg(-1=none) iters 0 [synth]; env BIG=1 = wide windows only (hi >= 4), EXT = range exponent
import numpy as np, pickle, sys, os
from bits import *

pin = sys.argv[1]
p, q = (list(P2), list(Q2)) if pin == "grid_a" else [list(v) for v in pickle.load(open(pin, "rb"))]
deg = int(sys.argv[3])
iters = int(sys.argv[4])
synth = None
if len(sys.argv) > 6 and sys.argv[6] == "synth":
    Ps = list(P2)
    Qs = list(Q2)
    Ps[9] = nextd(Ps[9], 3 * 1024)
    Ps[8] = nextd(Ps[8], -64)
    Qs[7] = nextd(Qs[7], 128)
    synth = (Ps, Qs)
X, Y = loadall(synth=synth)
st = Stair(X, Y)
WINB = {k: v for k, v in WIN.items() if v[1] >= 4} if os.environ.get("BIG") else WIN
for it in range(iters):
    m0, pats = patterns(p, q, st, win=WINB, ext=int(os.environ.get("EXT", "1")))
    w = st.nb[st.ok].astype(float)
    sw = np.sqrt(w)
    xc = (np.bincount(st.b, X)[st.ok] / st.nb[st.ok] - 2.5) * 2
    if deg >= 0:
        Qb, _ = np.linalg.qr(np.polynomial.legendre.legvander(xc, deg) * sw[:, None])
        proj = lambda v: v - Qb @ (Qb.T @ v)
    else:
        proj = lambda v: v
    r0 = proj(m0 * sw)
    pp = {k: (ts, np.array([proj(g * sw) for g in G])) for k, (ts, G) in pats.items()}
    c, sol = polish(r0, pp)
    for (wk, i), t in sol.items():
        arr = p if wk == "p" else q
        arr[i] = nextd(arr[i], t)
    print(it, "cost %.1f -> %.1f" % (float(r0 @ r0), c), {k: v for k, v in sol.items() if v}, flush=True)
    pickle.dump((p, q), open(sys.argv[2], "wb"))
    if all(v == 0 for v in sol.values()):
        break
