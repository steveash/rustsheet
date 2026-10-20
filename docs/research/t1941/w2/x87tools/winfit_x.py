"""Window fit under an x87 keep-structure with the LATE part frozen (only p[0..PE], q[0..QE] move).
usage: winfit_x.py start.pkl data.npz out.pkl KEEP rounds [truth.pkl -> synthetic Y from truth under KEEP]; env PE (4), QE (3)"""
import os, pickle, sys
import numpy as np
from search import dsearch, nextd
from x87lib import XScorer, xsim
p, q = [list(v) for v in pickle.load(open(sys.argv[1], 'rb'))]
dat = np.load(sys.argv[2]); X = np.ascontiguousarray(dat['X'], float); KEEP = [k for k in sys.argv[4].split(',') if k]
if len(sys.argv) > 6:
    pt, qt = [list(v) for v in pickle.load(open(sys.argv[6], 'rb'))]; Y = xsim(pt, qt, X, KEEP)
else:
    Y = np.ascontiguousarray(dat['Y'], float)
PE, QE = int(os.environ.get('PE', 4)), int(os.environ.get('QE', 3))
sc = XScorer(X, Y, KEEP); n = len(X); print('n', n, 'start %.4f' % (sc(p, q) / n), flush=True)
for rnd in range(int(sys.argv[5])):
    for which, last in (('p', PE), ('q', QE)):
        for kk in (1, 2, 3, 4):
            for off in range(last):
                if off + kk > last: continue
                for T, s in ((3000, 10), (30, 1)):
                    p, q, _ = dsearch(p, q, sc, which, kk, T, s, off)
    for which, last in (('p', PE), ('q', QE)):
        for i in range(last + 1):
            for R, stp in ((20000, 100), (120, 2), (6, 1)):
                arr = p if which == 'p' else q; orig = arr[i]; best = (sc(p, q), 0)
                for t in range(-R, R + 1, stp):
                    arr[i] = nextd(orig, t); v = sc(p, q)
                    if v > best[0]: best = (v, t)
                arr[i] = nextd(orig, best[1])
    print(rnd, '%.4f' % (sc(p, q) / n), flush=True)
    pickle.dump((p, q), open(sys.argv[3], 'wb'))
