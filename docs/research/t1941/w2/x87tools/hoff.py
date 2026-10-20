"""Per-run head-sum offsets (N s2, s3, s4; D d2, d3) by coordinate ascent on exact match, late chain = grid_a.
usage (repo root): python3 -I docs/research/t1941/w2/x87tools/hoff.py [truth.pkl] cases...   (first 8,192 doubles of each long run)
build: gcc -O2 -shared -fPIC -o libhoff.so hoff.c -lm (next to this file). hoffF.c adds fused bits fN/fD to hcount."""
import ctypes, sys, os, pickle, numpy as np
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'review3'))
from spec import load, p as P0, q as Q0
M3 = os.path.dirname(os.path.abspath(__file__)) + '/'
L = ctypes.CDLL(M3 + 'libhoff.so'); DP = ctypes.POINTER(ctypes.c_double); L.hcount.restype = ctypes.c_int
p, q = list(P0), list(Q0); pa = np.ascontiguousarray(p); qa = np.ascontiguousarray(q)
truth = None; args = sys.argv[1:]
if args[0].endswith('.pkl'): truth = pickle.load(open(args[0], 'rb')); args = args[1:]
def simy(pp, qq, X):
    LD = np.longdouble; sd = lambda v: v.astype(np.float64).astype(LD); x = X.astype(LD)
    n = LD(pp[0]) + 0 * x; d = LD(qq[0]) + 0 * x
    for c in pp[1:]: n = sd(sd(n * x) + LD(c))
    for c in qq[1:]: d = sd(sd(d * x) + LD(c))
    return sd(sd(x - LD(2)) * sd(n / d)).astype(np.float64)
KEYS = [('N', 2), ('N', 3), ('N', 4), ('D', 2), ('D', 3)]
for case in args:
    X0, Y0 = load(case)
    br = [0] + [i + 1 for i in range(len(X0) - 1) if np.nextafter(X0[i], np.inf) != X0[i + 1]] + [len(X0)]
    a, b = max(zip(br, br[1:]), key=lambda t: t[1] - t[0]); X = np.ascontiguousarray(X0[a:b][:8192]); Y = np.ascontiguousarray(Y0[a:b][:8192])
    if truth: Y = np.ascontiguousarray(simy(list(truth[0]), list(truth[1]), X))
    offN = np.zeros(10); offD = np.zeros(9)
    def sc():
        return L.hcount(pa.ctypes.data_as(DP), 10, qa.ctypes.data_as(DP), 9, offN.ctypes.data_as(DP), 4, offD.ctypes.data_as(DP), 3, X.ctypes.data_as(DP), Y.ctypes.data_as(DP), len(X)) / len(X)
    cur = sc(); start = cur
    for rnd in range(6):
        for ch, i in KEYS:
            arr = offN if ch == 'N' else offD; best = (cur, arr[i])
            for R, st in ((4.0, 1 / 8), (0.25, 1 / 64)):
                cen = best[1]
                for v in np.arange(cen - R, cen + R + 1e-12, st):
                    arr[i] = v; s = sc()
                    if s > best[0] + 1e-12: best = (s, v)
                arr[i] = best[1]
            cur = best[0]
    print('%-7s x0 %.4f n %d  start %.4f  fit %.4f  offsets N s2 %+.3f s3 %+.3f s4 %+.3f  D d2 %+.3f d3 %+.3f' % (case, X[0], len(X), start, cur, offN[2], offN[3], offN[4], offD[2], offD[3]), flush=True)
