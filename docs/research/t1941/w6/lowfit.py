# T-1941 W6: assign Excel's unexplained precise tracks (Lxl_tracks.json) to stage laws of a chain whose upper stages
# are pinned. For each unexplained track and each law key of stage_F (both chains, both signs), solve one weighted LS for
# the coefficient offsets m (ulps, all 19) under: function rows (func_rows, sigma SF), hold rows keeping the already
# explained tracks on their laws, and the candidate track (sigma ST) with one free constant (the track's integer).
# A real low-stage line fits to the track noise (~3e-7); a wrong law leaves a structured residual.
# Greedy joint mode: ADD="27:D:P2n1:+1,28:N:P2n1:+1" holds those tracks on those laws too; OUT=path writes the chain
# refitted on the function + all hold rows (offsets added as values, like applym.py).
# Usage: lowfit.py CHAIN TRACKS [top]
import sys, os, json, numpy as np, mpmath as mp

sys.path.insert(0, sys.path[0])
from synth import read_chain
from stagelaws import stage_F
from latsolve import func_rows

ST, SF, HOLD_DEV = 3e-7, 3e-16, 2e-6
N, D = read_chain(sys.argv[1])
N = list(np.array(N) / D[0])
D = list(np.array(D) / D[0])
T = json.load(open(sys.argv[2]))
top = int(sys.argv[3]) if len(sys.argv) > 3 else 3
nN = len(N)
cs = N + D[1:]
u = np.array([np.spacing(abs(v)) for v in cs])
H = 1e8  # finite-difference step in ulps: F is linear in the coefficients within a binade


def chains(m):
    c = np.array(cs) + m * u
    return c[:nN], np.concatenate([[1.0], c[nN:]])


def allF(x, m=None):
    Nm, Dm = chains(np.zeros(len(cs)) if m is None else m)
    return {"N": stage_F(Nm, x), "D": stage_F(Dm, x)}


def law_grad(x):
    """F per law at x for the chain, and dF/dm per law (finite differences)."""
    F0 = allF(x)
    G = {}
    for k in range(len(cs)):
        e = np.zeros(len(cs))
        e[k] = H
        F1 = allF(x, e)
        for ch in "ND":
            for key in F0[ch]:
                if key in F1[ch]:
                    d = ((F1[ch][key][0] - F0[ch][key][0] + 0.5) % 1) - 0.5
                    G.setdefault((ch, key), np.zeros((len(x), len(cs))))[:, k] = d / H
    return F0, G


def wrap(d):
    return ((d - np.median(d) + 0.5) % 1) - 0.5 + np.median(d)


# function rows (fixed)
fr = func_rows(N, D)
Af = np.array([[float(v) for v in g] for g, _ in fr]) / SF
bf = -np.array([float(v) for _, v in fr]) / SF
# explained tracks -> hold rows; unexplained -> candidates
prep = []
hold_A, hold_b = [], []
for i, t in enumerate(T):
    x = np.array(t["x"])
    Fo = np.array(t["F"])
    F0, G = law_grad(x)
    best = None
    for (ch, key), g in G.items():
        for sg in (1, -1):
            d = wrap(sg * Fo - F0[ch][key][0])
            dev = np.max(np.abs(d - np.round(np.median(d))))
            if best is None or dev < best[0]:
                best = (dev, ch, key, sg, d, g)
    forced = [a.split(":") for a in os.environ.get("ADD", "").split(",") if a]
    forced = {int(a[0]): (a[1], a[2], int(a[3])) for a in forced}
    if i in forced:
        ch, key, sg = forced[i]
        g = G[(ch, key)]
        d = wrap(sg * Fo - F0[ch][key][0])
        best = (0.0, ch, key, sg, d, g)
    if best[0] < HOLD_DEV or i in forced:
        dev, ch, key, sg, d, g = best
        hold_A.append(np.hstack([g, np.zeros((len(x), 1))]) / ST)
        hold_b.append((d - np.round(np.median(d))) / ST)
        print("hold track %2d: %s.%s %+d dev %.1e" % (i, ch, key, sg, dev))
    else:
        prep.append((i, x, Fo, F0, G))
Ah = np.vstack(hold_A)
bh = np.concatenate(hold_b)
Afz = np.hstack([Af, np.zeros((len(Af), 1))])
base_A = np.vstack([Afz, Ah])
base_b = np.concatenate([bf, bh])
sc = np.max(np.abs(np.vstack([base_A])), axis=0)
sc[sc == 0] = 1


def lsq(A, b):
    s = np.max(np.abs(A), axis=0)
    s[s == 0] = 1
    z, *_ = np.linalg.lstsq(A / s, b, rcond=None)
    return z / s


m0 = lsq(base_A, base_b)
chi0 = np.sum((base_A @ m0 - base_b) ** 2)
print("base: %d function + %d hold rows, chi2 %.1f" % (len(Af), len(bh), chi0))
print("hold rms per row (x ST): %.1e   function rms (x SF): %.1e" % (np.sqrt(np.mean((Ah @ m0 - bh) ** 2)) * ST, np.sqrt(np.mean((Afz @ m0 - bf) ** 2)) * SF))
if os.environ.get("OUT"):
    Nm, Dm = chains(np.round(m0[:-1]))
    open(os.environ["OUT"], "w").write("N " + " ".join(float(v).hex() for v in Nm) + "\nD " + " ".join(float(v).hex() for v in Dm) + "\n")
    print("m:", [int(v) for v in np.round(m0[:-1])])
for i, x, Fo, F0, G in prep:
    out = []
    for (ch, key), g in G.items():
        for sg in (1, -1):
            d = wrap(sg * Fo - F0[ch][key][0])
            At = np.hstack([g, np.ones((len(x), 1))]) / ST
            A = np.vstack([base_A, At])
            b = np.concatenate([base_b, d / ST])
            m = lsq(A, b)
            rt = (At @ m - d / ST) * ST
            dchi = np.sum((base_A @ m - base_b) ** 2) - chi0
            out.append((np.sqrt(np.mean(rt**2)), dchi, "%s.%s %+d" % (ch, key, sg)))
    out.sort()
    print(
        "track %2d (n=%2d, %.3f-%.3f): " % (i, len(x), x[0], x[-1])
        + "  ".join("%s rms %.1e dchi %.0f" % (h, r, dc) for r, dc, h in out[:top]),
        flush=True,
    )
