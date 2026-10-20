"""Per-op repair test. For a fitted (p, q) and data (X, Y): for each single op, recompute with that op left unstored
(80-bit) and count misses repaired minus hits broken. usage: oprep.py fit.pkl data.npz [KEEP for the fit, comma list]
Caveat (controls): within the stored-chain family the top op is specific (Nadd3-only truth flags Nadd3), but ANY early
rounding mismatch lights up the highest-weight early ops first, so read it as a localiser, not an identification."""
import pickle, sys
import numpy as np
from x87lib import xsim
p, q = [list(v) for v in pickle.load(open(sys.argv[1], 'rb'))]
dat = np.load(sys.argv[2]); X = dat['X']; Y = dat['Y']
base = [n for n in (sys.argv[3].split(',') if len(sys.argv) > 3 else []) if n]
y0 = xsim(p, q, X, base); miss = y0 != Y
print('n', len(X), 'hit %.4f' % (1 - miss.mean()), 'misses', int(miss.sum()))
ops = ['N%s%d' % (o, i) for i in range(1, len(p)) for o in ('mul', 'add')] + ['D%s%d' % (o, i) for i in range(1, len(q)) for o in ('mul', 'add')] + ['div']
res = []
for o in ops:
    if o in base: continue
    yo = xsim(p, q, X, base + [o])
    rep = int(((yo == Y) & miss).sum()); brk = int(((yo != Y) & ~miss).sum())
    res.append((rep - brk, rep, brk, int((yo != y0).sum()), o))
res.sort(reverse=True)
for r in res[:8]: print('%-7s net %6d repaired %6d broken %6d flips %6d' % (r[4], r[0], r[1], r[2], r[3]))
