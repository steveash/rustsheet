import ctypes, sys, os, numpy as np
from mpmath import mp, mpf, matrix, lu_solve
from multiprocessing import Pool
mp.dps = 50
sys.path.insert(0, '/home/user/rustsheet/docs/research/t1941/review3'); os.chdir('/home/user/rustsheet')
from spec import load, p as P0, q as Q0
L = ctypes.CDLL(os.environ.get('LIBHOFF', './libhoff.so')); DP = ctypes.POINTER(ctypes.c_double); L.hcount.restype = ctypes.c_int
xs = [mpf(2) + mpf(i) / 60 for i in range(61)]
def nd(v, k):
    i = np.frombuffer(np.float64(v).tobytes(), np.int64)[0] + (k if v >= 0 else -k); return float(np.frombuffer(np.int64(i).tobytes(), np.float64)[0])
def resolve(base, deg, idx, k):
    c = list(base); c[idx] = nd(c[idx], k); free = list(range(idx + 1, deg + 1))
    tgt = [sum(mpf(base[i]) * x ** (deg - i) for i in range(deg + 1)) - sum(mpf(c[i]) * x ** (deg - i) for i in range(idx + 1)) for x in xs]
    A = [[x ** (deg - i) for i in free] for x in xs]; n = len(free); M = matrix(n, n); v = matrix(n, 1)
    for r in range(len(xs)):
        for i in range(n):
            v[i] += A[r][i] * tgt[r]
            for j in range(n): M[i, j] += A[r][i] * A[r][j]
    sol = lu_solve(M, v)
    for kk, i in enumerate(free): c[i] = float(sol[kk])
    return c
RUNS = os.environ.get('RUNS', 'r112 r901 r113 r703').split(); NPT = int(os.environ.get('NPT', 1024))
def simy(pp, qq, X):
    LD = np.longdouble; sd = lambda v: v.astype(np.float64).astype(LD); x = X.astype(LD)
    n = LD(pp[0]) + 0 * x; d = LD(qq[0]) + 0 * x
    for c in pp[1:]: n = sd(sd(n * x) + LD(c))
    for c in qq[1:]: d = sd(sd(d * x) + LD(c))
    return sd(sd(x - LD(2)) * sd(n / d)).astype(np.float64)
DATA = []
for case in RUNS:
    X0, Y0 = load(case)
    br = [0] + [i + 1 for i in range(len(X0) - 1) if np.nextafter(X0[i], np.inf) != X0[i + 1]] + [len(X0)]
    a, b = max(zip(br, br[1:]), key=lambda t: t[1] - t[0]); DATA.append((np.ascontiguousarray(X0[a:b][:NPT]), np.ascontiguousarray(Y0[a:b][:NPT])))
PLANT = os.environ.get('PLANT')
if PLANT:
    nm, k = PLANT.split(':'); tp = resolve(P0, 9, int(nm[1:]), int(k)) if nm[0] == 'p' else list(P0); tq = resolve(Q0, 8, int(nm[1:]), int(k)) if nm[0] == 'q' else list(Q0)
    DATA = [(X, np.ascontiguousarray(simy(tp, tq, X))) for X, Y in DATA]
Z10 = np.zeros(10); Z9 = np.zeros(9)
CH, IDX = sys.argv[1][0], int(sys.argv[1][1:]); lo, hi, st = int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
def score(k):
    pc = np.ascontiguousarray(resolve(P0, 9, IDX, k) if CH == 'p' else P0); qc = np.ascontiguousarray(resolve(Q0, 8, IDX, k) if CH == 'q' else Q0)
    per = [L.hcount(pc.ctypes.data_as(DP), 10, qc.ctypes.data_as(DP), 9, Z10.ctypes.data_as(DP), 0, Z9.ctypes.data_as(DP), 0, X.ctypes.data_as(DP), Y.ctypes.data_as(DP), len(X)) / len(X) for X, Y in DATA]
    return float(np.mean(per))
ks = list(range(lo, hi + 1, st))
with Pool(4) as pool: res = pool.map(score, ks)
o = np.argsort(res)[::-1]
print('%s%d scan [%d,%d] step %d plant=%s: k=0 %.4f ; top: %s' % (CH, IDX, lo, hi, st, PLANT, res[ks.index(0)] if 0 in ks else float('nan'), ' '.join('%d:%.4f' % (ks[j], res[j]) for j in o[:6])), flush=True)
