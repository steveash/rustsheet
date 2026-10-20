"""Per-run hand-off offset meter (T-1941 W2). Model = committed grid_a. For each long run, best constant offset (dN, dD)
at the pre-rounding sums P4+p4 / P3+q3 (grid +-G/R ulps), exact rate at 0 and at best.
usage (repo root): python3 -I docs/research/t1941/review3/offm.py [truth.pkl | ctl:p3:+5,q2:-7] cases...
(truth given -> synthetic Y from that stored-Horner truth). Build liboffm.so next to this file first.
Controls: p4 +2 -> 1.000; p3/p2/q2 head moves -> 0.91-0.93; function-preserving early-only moves -> 0.92-0.96 on every
run; late/cross-N/D moves -> limited, erratic. REAL 14 runs: 0.66-0.78 -> 0.71-0.83."""
import ctypes, os, pickle, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
L = ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'liboffm.so')); DP = ctypes.POINTER(ctypes.c_double)
G, R = int(os.environ.get('G', 80)), int(os.environ.get('R', 32))
p = list(P0); q = list(Q0); truth = None; args = sys.argv[1:]
def nd(v, k):
    for _ in range(abs(k)): v = float(np.nextafter(v, np.inf if (k > 0) == (v >= 0) else -np.inf))
    return v
if args and args[0].endswith('.pkl'):
    truth = tuple(list(v) for v in pickle.load(open(args[0], 'rb'))); args = args[1:]
elif args and args[0].startswith('ctl:'):
    pt, qt = list(p), list(q)
    for it in args[0][4:].split(','):
        nm, k = it.split(':'); a = pt if nm[0] == 'p' else qt; a[int(nm[1:])] = nd(a[int(nm[1:])], int(k))
    truth = (pt, qt); args = args[1:]
def simy(pp, qq, X):
    LD = np.longdouble; sd = lambda v: v.astype(np.float64).astype(LD); x = X.astype(LD)
    n = LD(pp[0]) + 0 * x; d = LD(qq[0]) + 0 * x
    for c in pp[1:]: n = sd(sd(n * x) + LD(c))
    for c in qq[1:]: d = sd(sd(d * x) + LD(c))
    return sd(sd(x - LD(2)) * sd(n / d)).astype(np.float64)
pa = np.ascontiguousarray(p); qa = np.ascontiguousarray(q)
for case in args:
    X0, Y0 = load(case)
    br = [0] + [i + 1 for i in range(len(X0) - 1) if np.nextafter(X0[i], np.inf) != X0[i + 1]] + [len(X0)]
    a, b = max(zip(br, br[1:]), key=lambda t: t[1] - t[0]); X = np.ascontiguousarray(X0[a:b]); Y = np.ascontiguousarray(Y0[a:b])
    if truth: Y = np.ascontiguousarray(simy(*truth, X))
    W = 2 * G + 1; c = np.zeros(W * W, np.int64)
    L.offm(pa.ctypes.data_as(DP), 10, qa.ctypes.data_as(DP), 9, 4, 3, X.ctypes.data_as(DP), Y.ctypes.data_as(DP), len(X), G, R, c.ctypes.data_as(ctypes.POINTER(ctypes.c_longlong)))
    c = c.reshape(W, W) / len(X); i, j = np.unravel_index(np.argmax(c), c.shape)
    print('%-7s x0 %.4f n %5d  at0 %.4f  best %.4f dN %+.3f dD %+.3f' % (case, X[0], len(X), c[G, G], c[i, j], (i - G) / R, (j - G) / R), flush=True)
