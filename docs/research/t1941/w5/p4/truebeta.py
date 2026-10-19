"""True per-segment mean stage rounding errors (truth minus c5), in ulps of S, for the control (validation of linreader)."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from sc import candidate, load
from events import events
L = np.longdouble
def st(v): return np.asarray(v, dtype=L).astype(np.float64)
def stage_errs(C, x):
    s = np.full_like(x, C[-1]); out = {}
    for j in range(len(C)-2, -1, -1):
        p = st(L(s)*L(x)); ex = L(p)+L(C[j]); s = st(ex); out[j] = ((L(s)-ex)/L(np.spacing(np.abs(s)))).astype(float)
    return out
x, b, c = load(); m = (x >= 0.7) & (x < 1.47) & (np.abs(x-1) > 2e-3); x = x[m]
N5 = candidate('N3'); D5 = candidate('D3'); Nt = N5.copy(); Dt = D5.copy()
spec = {('N', 0): 37, ('D', 1): -11, ('D', 2): 3}
for (ch, k), j in spec.items():
    C = Nt if ch == 'N' else Dt; C[k] += j*np.spacing(abs(C[k]))
ev = events(N5, D5, 0.7, 1.47)
for ch, C0, C1 in (('N', N5, Nt), ('D', D5, Dt)):
    e0 = stage_errs(C0, x); e1 = stage_errs(C1, x)
    for j in ([0] if ch == 'N' else [1, 2]):
        cuts = sorted(xe for xe, nm, a, bb in ev if nm in ('%s.S%d' % (ch, j), '%s.P%d' % (ch, j)))
        seg = np.searchsorted(cuts, x); dmean = [np.mean(e1[j][seg == s]-e0[j][seg == s]) for s in range(len(cuts)+1)]
        # include the constant's own move (in ulps of S on each segment): effective error relative to exact c
        print('%s.S%d cuts %s' % (ch, j, ' '.join('%.4f' % v for v in cuts)))
        print('   mean err(truth)-err(c5) per segment (ulps S):', ' '.join('%+.3f' % v for v in dmean), '| minus seg0:', ' '.join('%+.3f' % (v-dmean[0]) for v in dmean[1:]))
