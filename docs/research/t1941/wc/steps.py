# Excel and candidate-sim step heights at given crossings (run from review/, uses k1lib). args: cand.txt c1,c2,...
import sys

sys.path.insert(0, ".")
from k1lib import *
import mpmath as mp

N, D = load_cand(sys.argv[1])
y, g = load_rows("g_rows.txt")
cross = [float(v) for v in sys.argv[2].split(",")]
W = [0.002, 0.005]
sel = np.zeros(len(y), bool)
for c in cross:
    sel |= np.abs(y - c) < max(W)
y = y[sel]
g = g[sel]
mp.mp.dps = 25
gam = np.array([float(mp.gamma(mp.mpf(v))) for v in y])
m = G(N, D, y)
u = np.spacing(np.abs(g))
ex = (g - gam) / u
sm = (m - gam) / u
for c in cross:
    out = []
    for w in W:
        L_ = (y >= c - w) & (y < c)
        R_ = (y > c) & (y <= c + w)
        if L_.sum() < 30 or R_.sum() < 30:
            out.append("w=%.3f n=%d/%d" % (w, L_.sum(), R_.sum()))
            continue
        se = np.sqrt(ex[L_].var() / L_.sum() + ex[R_].var() / R_.sum())
        out.append(
            "w=%.3f Excel step %+.3f (se %.3f) sim step %+.3f"
            % (w, ex[R_].mean() - ex[L_].mean(), se, sm[R_].mean() - sm[L_].mean())
        )
    print("crossing %.4f: " % c + " | ".join(out), flush=True)
