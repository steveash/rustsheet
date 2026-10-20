"""Per-run stage reads (T-1941 W5 part 5): within a run (consecutive doubles, tiny x span) a smooth stage error is
constant, so summing per-point hit sets over the run's points peaks at the run's (dn, dd) = shifts of the stored
N.S_JN and D.S_JD that reproduce Excel. Reports the peak, its margin, and the null (B shifted by 1 ulp).
usage: runread.py SET JN JD K   env: RL (points per run used, 4096), NRUN (all), OUT"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from meter import L, st
N, D = ld(sys.argv[1]); JN, JD, K = int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
z = np.load('/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz')
x, b = z['x'], z['b']
br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0]+1; segs = np.split(np.arange(len(x)), br)
RL = int(os.environ.get('RL', 4096)); segs = [s[:RL] for s in segs if len(s) >= 256]
nr = int(os.environ.get('NRUN', 0)); segs = segs[:nr] if nr else segs
idx = np.concatenate(segs); x, b = x[idx], b[idx]; rid = np.repeat(np.arange(len(segs)), [len(s) for s in segs])
xl = L(x); t = L(st(xl-L(1.0)))
def upper(C, J):
    s = L(C[-1])
    for j in range(len(C)-2, J-1, -1): s = L(st(L(st(s*xl))+L(C[j])))
    return s.astype(np.float64)
def lower(C, J, sJ):
    s = L(sJ)
    for j in range(J-1, -1, -1): s = L(st(L(st(s*xl))+L(C[j])))
    return s
def shift(v, k):
    r = v.copy()
    for _ in range(abs(k)): r = np.nextafter(r, np.inf if k > 0 else -np.inf)
    return r
sn, sd = upper(N, JN), upper(D, JD)
ln = {k: lower(N, JN, shift(sn, k)) for k in range(-K, K+1)}; ldd = {k: lower(D, JD, shift(sd, k)) for k in range(-K, K+1)}
M = 2*K+1; nR = len(segs)
def table(bb):
    T = np.zeros((nR, M, M))
    for a in range(-K, K+1):
        for c in range(-K, K+1):
            T[:, a+K, c+K] = np.bincount(rid, weights=(st(t*L(st(ln[a]/ldd[c]))) == bb).astype(float), minlength=nR)
    return T/np.bincount(rid)[:, None, None]
T = table(b); Tn = table(np.nextafter(b, np.inf))
xm = np.array([x[rid == r][0] for r in range(nR)])
res = []
for r in range(nR):
    f = T[r].ravel(); o = np.argsort(f)[::-1]; fn = Tn[r].max()
    res.append((xm[r], o[0]//M-K, o[0] % M-K, f[o[0]], f[o[1]], fn))
res = np.array(res)
print('%d runs; peak hit-rate median %.3f (2nd best %.3f), null (B+1ulp) best median %.3f; runs with peak >= 0.99: %d'
      % (nR, np.median(res[:, 3]), np.median(res[:, 4]), np.median(res[:, 5]), int((res[:, 3] >= 0.99).sum())))
for q in np.argsort(res[:, 0])[::max(1, nR//25)]:
    print('  x %.6f  dn %+d dd %+d  hit %.3f (2nd %.3f, null %.3f)' % tuple(res[q]))
if os.environ.get('OUT'): np.save(os.environ['OUT'], res)
