"""Group c5-unmatched data peaks into tracks, then test which c5-predicted line (single, 2x, sum/diff of two)
follows each whole track (full-track test). Tolerance 4e-4 cycles/step."""
import numpy as np, sys, os, itertools
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import peaks as PK
from predict import predict, fold
from sc import res
L = 2048; rows = []
for s in PK.segs:
    xx = PK.x[s]; bb = PK.b[s]; sim = res(xx, PK.N, PK.D)
    pd = PK.peaks(PK.resid(xx, bb)); ps = PK.peaks(PK.resid(xx, sim), 10)
    fs = np.array([f for f, _ in ps]) if ps else np.array([9.])
    for f, p in pd:
        if np.min(np.abs(fs-f)) > 3.0/L: rows.append((xx[L//2], f, p))
rows = np.array(rows)
T = {'T1': (rows[:, 0] < 1) & (rows[:, 1] > 0.305) & (rows[:, 1] < 0.395) & (np.abs(rows[:, 1]-(0.3134+0.323*(rows[:, 0]-0.7413))) < 0.004),
     'T2': (rows[:, 0] < 0.8) & (rows[:, 1] > 0.336) & (rows[:, 1] < 0.353),
     'T3': (rows[:, 0] > 1.3)}
for name, m in T.items():
    pts = rows[m]; print(name, 'points', len(pts), ' '.join('%.4f:%.4f' % (a, b) for a, b, _ in pts[np.argsort(pts[:, 0])]))
    pred = [predict(xv) for xv in pts[:, 0]]
    names = [n for n, _ in pred[0]]
    V = np.array([[f for _, f in pr] for pr in pred])          # points x lines
    cands = {}
    for i, n in enumerate(names):
        cands[n] = V[:, i]; cands['2*'+n] = np.array([fold(2*v) for v in V[:, i]])
    for i, j in itertools.combinations(range(len(names)), 2):
        for sg in (1, -1):
            cands['%s %s %s' % (names[i], '+' if sg > 0 else '-', names[j])] = np.array([fold(a+sg*b) for a, b in zip(V[:, i], V[:, j])])
    sc = sorted(((np.mean(np.abs(v-pts[:, 1]) < 4e-4), np.median(np.abs(v-pts[:, 1])), k) for k, v in cands.items()), reverse=True)[:6]
    for frac, med, k in sc: print('   %-40s match %.2f  median |df| %.1e' % (k, frac, med))
