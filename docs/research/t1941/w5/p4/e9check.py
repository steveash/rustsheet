"""Independent check on the e9xx anchor ladders (not used in any fit): per-run mean relative bias of Excel minus
candidate sim, grouped by anchor. Correct plateau readings make the per-run means agree (small spread) and near
zero up to the smooth function difference."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, os.path.join(HERE, '..'))
from sc import candidate, res
X = []; B = []
for l in open(sys.argv[1]):
    xh, bh, _, _, nm = l.split()
    if bh != 'nan' and nm.startswith('kmyst-gam-e9'): X.append(float.fromhex(xh)); B.append(float.fromhex(bh))
X = np.array(X); B = np.array(B); o = np.argsort(X); X = X[o]; B = B[o]
br = np.where(np.nextafter(X[:-1], np.inf) != X[1:])[0]+1; segs = np.split(np.arange(len(X)), br)
N5 = candidate('N3'); D5 = candidate('D3'); XL = X.astype(np.longdouble)
ref = (XL-1)*np.polyval(N5.astype(np.longdouble)[::-1], XL)/np.polyval(D5.astype(np.longdouble)[::-1], XL); U = np.abs(ref)*np.longdouble(2.0**-52)
def ld(f):
    l = open(f).read().split('\n'); return np.array([float.fromhex(v) for v in l[0].split()]), np.array([float.fromhex(v) for v in l[1].split()])
anchors = [0.75, 0.875, 1.125, 1.25, 1.375]
print('runs', len(segs))
for nm, (N, D) in [('c5', (N5, D5))] + [(f, ld(os.path.join(HERE, f))) for f in sys.argv[2:]]:
    e = ((B-res(X, N, D))/U).astype(float); out = []
    for a in anchors:
        mu = np.array([e[s].mean() for s in segs if abs(X[s[0]]-a) < 0.01])
        out.append('%.3f: mean %+.2f spread %.2f (n %d)' % (a, mu.mean(), mu.std(), len(mu)))
    print('%-18s' % nm, ' | '.join(out))
