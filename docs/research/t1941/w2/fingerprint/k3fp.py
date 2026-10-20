import sys, numpy as np
sys.path.insert(0, 'docs/research/t1941/w2/fingerprint'); sys.path.insert(0, 'docs/research/t1941/w2/k3')
from finger import *
import k3lib
x, b, _ = k3lib.rows(sub=400000, seed=9)
pl, ql = k3lib.ldset('docs/research/t1941/w5/p4/ss_fc.txt'); P = pl[::-1]; Q = ql[::-1]   # high-first
def stored_below(k, deg=9): return ops_from(1, deg - k) if k <= deg else frozenset()   # low-first indices >= k unstored
S = {'all stored': (frozenset(), frozenset()), 'm3': (stored_below(3), stored_below(3)), 'm2': (stored_below(2), ops_from(1, 9)),
     'stored below 5': (stored_below(5), stored_below(5)), 'top-2 unstored': (stored_below(7), stored_below(7))}
_, _, rr, mr = fit(x, b, 1.0, P, Q, verbose=True); print('K3 REAL rms %.3f median %.3f' % (rr, mr))
for nm, (uN, uD) in S.items():
    _, _, r, m = fit(x, sim(x, 1.0, P, Q, uN, uD), 1.0, P, Q)
    print('K3 PLANTED %-16s rms %.3f median %.3f' % (nm, r, m), flush=True)
