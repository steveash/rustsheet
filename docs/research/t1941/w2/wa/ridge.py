# args: table start_csv synth|real T; walks the phase-flat ridge, exact-scores on a 20k subsample
exec(open("phasecore.py").read())
from bits import loadall, Scorer, nextd

M = np.vstack([gg1, gg2]).T * np.sqrt(gw)[:, None]
_, sv, vt = np.linalg.svd(M)
rd = vt[-1] / abs(vt[-1][0])
start = np.array([float(v) for v in sys.argv[2].split(",")])
synth = [list(v) for v in pickle.load(open("../w/synC.pkl", "rb"))] if sys.argv[3] == "synth" else None
X, Y = loadall(synth=synth)
sel = np.random.default_rng(0).choice(len(X), 20000, replace=False)
sc = Scorer(X[sel], Y[sel])


def mk(dc):
    pp = list(p)
    qq = list(q)
    for (w, i), d in zip(WHICH, dc):
        a = pp if w == "p" else qq
        a[i] = nextd(a[i], int(d))
    return pp, qq


best = (0, None)
Tn = int(sys.argv[4])
for t in range(-Tn, Tn + 1):
    dp1 = start[ip1] + t
    dq0 = round(start[iq0] + t * rd[1])
    dc = np.round(dp1 * w1 + dq0 * w2)
    dc[ip1] = dp1
    dc[iq0] = dq0
    dc, c = refine(dc, (-2, -1, 1, 2))
    if c < 0.95:
        continue
    s = sc(*mk(dc)) / len(sel)
    if s > best[0]:
        best = (s, dc.copy())
        print("t", t, "coh %.4f exact %.4f" % (c, s), [int(v) for v in dc], flush=True)
pickle.dump(best[1], open(sys.argv[1] + ".ridge", "wb"))
