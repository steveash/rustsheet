import sys, pickle, numpy as np
sys.path.insert(0, 'docs/research/t1941/w2/fingerprint')
from finger import *
z = np.load('k2rows.npz'); X_, B_ = z['x'], z['b']          # from: mkrange.py 2 3 k2rows.npz right
P, Q = pickle.load(open('r3set.pkl', 'rb'))                  # (P, Q) high-first, the R3-basin hex
S = {'all stored': (frozenset(), frozenset()),
     'M* (N S3, D S2)': (frozenset({('S', 3)}), frozenset({('S', 2)})),
     'head-80 thru s3/d2': (ops_from(1, 3), ops_from(1, 2)),
     'everything 80': (ops_from(1, 9), ops_from(1, 8))}
rng = np.random.default_rng(5)
for nm, m in (('far [2.1,3]', X_ >= 2.1), ('near-2 (2,2.1)', X_ < 2.1)):
    i = np.sort(rng.choice(np.flatnonzero(m), min(400000, m.sum()), replace=False)); x, b = X_[i], B_[i]
    _, _, r0, m0 = fit(x, b, 2.0, P, Q, verbose=True); print('%s REAL rms %.3f median %.3f' % (nm, r0, m0))
    for pl, (uN, uD) in S.items():
        _, _, r, mm = fit(x, sim(x, 2.0, P, Q, uN, uD), 2.0, P, Q)
        print('%s   PLANTED %-20s rms %.3f median %.3f' % (nm, pl, r, mm), flush=True)
