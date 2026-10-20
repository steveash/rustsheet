"""Per-point stage reads (T-1941 W5 part 5; W2's K2 route): with the late chain fixed (the stages below N.S_JN and
D.S_JD, from SET), find per point the shifts (dn, dd) in ulps of the stored N.S_JN and D.S_JD values, |dn|,|dd| <= K,
that reproduce Excel's B. Reports the explained fraction and the ambiguity; saves per-point read intervals.
usage: stageread.py SET JN JD K [NPZ]   env: NSUB, OUT (npz of reads)"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from meter import L, st
N, D = ld(sys.argv[1]); JN, JD, K = int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
z = np.load(sys.argv[5] if len(sys.argv) > 5 else '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz')
x, b = z['x'], z['b']
ns = int(os.environ.get('NSUB', 0))
if ns: i = np.random.default_rng(1).choice(len(x), ns, replace=False); x, b = x[i], b[i]
xl = L(x); t = L(st(xl-L(1.0)))
def upper(C, J):            # stored S_J
    s = L(C[-1])
    for j in range(len(C)-2, J-1, -1):
        s = L(st(L(st(s*xl))+L(C[j])))
    return s.astype(np.float64)
def lower(C, J, sJ):        # from stored S_J down to S_0
    s = L(sJ)
    for j in range(J-1, -1, -1):
        s = L(st(L(st(s*xl))+L(C[j])))
    return s
sn, sd = upper(N, JN), upper(D, JD)
def shift(v, k):
    r = v.copy()
    for _ in range(abs(k)): r = np.nextafter(r, np.inf if k > 0 else -np.inf)
    return r
ln = {k: lower(N, JN, shift(sn, k)) for k in range(-K, K+1)}
ld_ = {k: lower(D, JD, shift(sd, k)) for k in range(-K, K+1)}
hit = np.zeros((len(x), 2*K+1, 2*K+1), bool)
for a in range(-K, K+1):
    for c in range(-K, K+1):
        hit[:, a+K, c+K] = st(t*L(st(ln[a]/ld_[c]))) == b
anyh = hit.any((1, 2)); base = hit[:, K, K]
nsol = hit.sum((1, 2))
print('SET %s, N.S%d / D.S%d, K %d, %d pts: base exact %.4f, explained %.4f; median #solutions among explained %d'
      % (sys.argv[1].split('/')[-1], JN, JD, K, len(x), base.mean(), anyh.mean(), int(np.median(nsol[anyh])) if anyh.any() else 0))
# marginal: fraction explained with dd = 0 only / dn = 0 only
print('  explained with dd=0 only: %.4f; with dn=0 only: %.4f' % (hit[:, :, K].any(1).mean(), hit[:, K, :].any(1).mean()))
if os.environ.get('OUT'):
    np.savez(os.environ['OUT'], x=x, hit=np.packbits(hit.reshape(len(x), -1), axis=1), K=K, sn=sn, sd=sd)
