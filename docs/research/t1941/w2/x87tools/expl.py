"""Which single +-1 stage re-decision explains Excel's misses (late chain grid_a; D allowed -1/0/+1 at d3).
usage (repo root): python3 -I docs/research/t1941/w2/x87tools/expl.py cases..."""
import sys, os, numpy as np
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'review3'))
from spec import load, p, q
LD = np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a = np.abs(v.astype(np.float64)); return (np.nextafter(a, np.inf) - a).astype(LD)
def chain(c, x, shift_at=None, shift=0, keepmul=None):
    s = LD(c[0]) + 0 * x
    for i in range(1, len(c)):
        P = s * x
        if keepmul != i: P = sd(P)
        s = sd(P + LD(c[i]))
        if shift_at == i and shift: s = sd(s + shift * ulp(s))
    return s
for case in sys.argv[1:]:
    X0, Y0 = load(case)
    br = [0] + [i + 1 for i in range(len(X0) - 1) if np.nextafter(X0[i], np.inf) != X0[i + 1]] + [len(X0)]
    a, b = max(zip(br, br[1:]), key=lambda t: t[1] - t[0]); x = X0[a:b].astype(LD); Y = Y0[a:b]; t = sd(x - LD(2))
    Ds = [chain(q, x, 3, sh) for sh in (0, -1, 1)]
    def ok(n): return np.any([sd(t * sd(n / d)).astype(np.float64) == Y for d in Ds], axis=0)
    def okD0(n): return sd(t * sd(n / Ds[0])).astype(np.float64) == Y
    base = okD0(chain(p, x)); miss = ~base
    print('==', case, 'hit %.3f' % base.mean(), 'misses %d' % miss.sum())
    print('   D+-1 alone fixes %.3f of misses' % ok(chain(p, x))[miss].mean())
    for k in range(1, 7):
        e = np.zeros(len(Y), bool)
        for sh in (-1, 1): e |= ok(chain(p, x, k, sh))
        print('   N s%d +-1 (with D in {-1,0,1}) explains %.3f of misses' % (k, e[miss].mean()))
    for km in (3, 4, 5):
        n = chain(p, x, keepmul=km); print('   Nmul%d unstored: repairs %.3f of misses, breaks %.3f of hits' % (km, okD0(n)[miss].mean(), (~okD0(n))[base].mean()))
