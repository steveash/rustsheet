import pickle, sys, os, numpy as np
from fpylll import IntegerMatrix, LLL
from multiprocessing import Pool
sys.path.insert(0, '/home/user/rustsheet/docs/research/t1941/review3'); os.chdir('/home/user/rustsheet')
from spec import load, p as P0, q as Q0
LD = np.longdouble
def sd(v): return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)
def nd(v, k):
    i = np.frombuffer(np.float64(v).tobytes(), np.int64)[0] + (k if v >= 0 else -k); return float(np.frombuffer(np.int64(i).tobytes(), np.float64)[0])
IDXN = eval(os.environ.get('IDXN', '[5,6,7,8,9]')); IDXD = eval(os.environ.get('IDXD', '[4,5,6,7,8]'))
cur = [list(v) for v in pickle.load(open(os.environ['START'], 'rb'))]
def lattice(c, idx, deg):
    xs = np.linspace(2, 3, 25); val = np.array([sum(c[i] * x ** (deg - i) for i in range(deg + 1)) for x in xs]); un = np.spacing(np.abs(val))
    F = np.array([[np.spacing(abs(c[i])) * x ** (deg - i) / u for x, u in zip(xs, un)] for i in idx])
    S = 2 ** 20; n = len(idx); M = IntegerMatrix(n, n + len(xs))
    for r in range(n):
        M[r, r] = 1
        for j in range(len(xs)): M[r, n + j] = int(round(F[r, j] * S / 64))
    LLL.reduction(M)
    vecs = [np.array([M[r, k] for k in range(n)]) for r in range(n)]
    return [(v, float(np.max(np.abs(v @ F)))) for v in vecs]
LN = lattice(cur[0], IDXN, 9); LDn = lattice(cur[1], IDXD, 8)
for nm, L in (('N', LN), ('D', LDn)):
    print(nm); [print('   ', v, 'max effect %.2f ulp' % e) for v, e in L]
CL = []
for nn in range(1, 17):
    for h in 'ab':
        X0, Y0 = load('q2%02d%s' % (nn, h))
        br = [0] + [i + 1 for i in range(len(X0) - 1) if np.nextafter(X0[i], np.inf) != X0[i + 1]] + [len(X0)]
        for k, (a, b) in enumerate(zip(br, br[1:])):
            if k % 2 == 0: CL.append((X0[a:b][:192].astype(LD), Y0[a:b][:192]))
XC = np.concatenate([c[0] for c in CL]); YC = np.concatenate([c[1] for c in CL])
LR = []
for case in 'r701 r112 p401 r702 r610 r901 r703 r113 r704 r611 r705 v401a v404a v407a'.split():
    X0, Y0 = load(case); br = [0] + [i + 1 for i in range(len(X0) - 1) if np.nextafter(X0[i], np.inf) != X0[i + 1]] + [len(X0)]
    a, b = max(zip(br, br[1:]), key=lambda t: t[1] - t[0]); LR.append((X0[a:b][:2048].astype(LD), Y0[a:b][:2048]))
UN = eval(os.environ.get('UN', "{'N': {2: 'U', 3: 'U'}, 'D': {2: 'P'}}"))   # 'U' = whole stage 80-bit, 'P' = product stored, add unstored
def chs(c, x, side):
    s = LD(c[0]) + 0 * x; m = UN[side]
    for i in range(1, len(c)):
        md = m.get(i, 'S')
        if md == 'U': s = s * x + LD(c[i]); continue
        P = sd(s * x); s = P + LD(c[i]) if md == 'P' else sd(P + LD(c[i]))
    return s
def y(pq, X): return sd(sd(X - LD(2)) * sd(chs(pq[0], X, 'N') / chs(pq[1], X, 'D'))).astype(np.float64)
def apply(pq, side, idx, v, m):
    c = [list(pq[0]), list(pq[1])]
    for i, k in zip(idx, v): c[side][i] = nd(c[side][i], int(m * k))
    return c
MOVES = [(0, IDXN, v) for v, e in LN] + [(1, IDXD, v) for v, e in LDn]
if os.environ.get('PLANT'):
    truth = apply(apply(cur, 0, IDXN, LN[2][0], 2), 1, IDXD, LDn[2][0], -1); YC = y(truth, XC); LR = [(X, y(truth, X)) for X, _ in LR]
    print('PLANTED: N 2*v2, D -1*v2', flush=True)
def scc(pq): return float(np.mean(y(pq, XC) == YC))
def hold(pq): return float(np.mean([np.mean(y(pq, X) == Y) for X, Y in LR]))
best = scc(cur); print('start clusters %.4f holdout %.4f' % (best, hold(cur)), flush=True)
MULT = [1, -1, 2, -2, 3, -3, 5, -5, 8, -8]
with Pool(4) as pool:
    for rnd in range(int(os.environ.get('RND', 8))):
        cands = [apply(cur, s, idx, v, m) for s, idx, v in MOVES for m in MULT]
        for a in range(len(MOVES)):
            for b in range(a + 1, len(MOVES)):
                if MOVES[a][0] != MOVES[b][0]: continue
                for ma in (1, -1):
                    for mb in (1, -1): cands.append(apply(apply(cur, *MOVES[a], ma), *MOVES[b], mb))
        r = pool.map(scc, cands, chunksize=4); j = int(np.argmax(r))
        if r[j] <= best + 1e-4: print('round %d: no improvement (%.4f)' % (rnd, best)); break
        best = r[j]; cur = cands[j]; print('round %d: clusters %.4f holdout %.4f' % (rnd, best, hold(cur)), flush=True)
pickle.dump(cur, open('nullfit5%s_%s.pkl' % ('_plant' if os.environ.get('PLANT') else '', os.environ.get('TAG', '')), 'wb'))
print('p hex', [v.hex() for v in cur[0]]); print('q hex', [v.hex() for v in cur[1]])
