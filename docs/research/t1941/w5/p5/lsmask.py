"""Function-level LS refit of the head under a mask (T-1941 W5 part 5; the coordinator's quick check). Free: N3..N9,
D4..D9 (late part fixed). Gauss-Newton on the output residual in output ulps, with the masked model; smooth
sensitivities (relative change x^j/N or -x^j/D). Coefficients rounded to doubles each step. Reports exact before/after.
usage: lsmask.py NPZ   env: NSUB (400000), IT (4)"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from meter import L, st
from maskmodel import res
N0, D0 = ld(os.path.join(HERE, '..', 'p4', 'ss_fc.txt'))
z = np.load(sys.argv[1]); i = np.random.default_rng(3).choice(len(z['x']), int(os.environ.get('NSUB', 400000)), replace=False)
x, b = z['x'][i], z['b'][i]; ok = np.abs(x-1) > 1e-3; x, b = x[ok], b[ok]
FN, FD = list(range(3, 10)), list(range(4, 10))
XL = x.astype(L)
for kn, kd, nm in ((99, 99, 'stored'), (3, 3, 'm3 (stored <3 both)'), (2, 0, 'm2 (N<2, D 80-bit)')):
    N, D = N0.copy(), D0.copy(); e0 = np.mean(res(x, N, D, kn, kd) == b)
    for it in range(int(os.environ.get('IT', 4))):
        y = res(x, N, D, kn, kd); u = np.spacing(np.abs(y))
        r = ((b-y)/u)                                  # output ulps
        Nv = np.polyval(N.astype(L)[::-1], XL); Dv = np.polyval(D.astype(L)[::-1], XL); yv = np.abs(y)/u  # |y| in ulps
        cols = [(np.spacing(abs(N[j]))*XL**j/Nv).astype(float)*yv*np.sign(y) for j in FN] + \
               [(-np.spacing(abs(D[j]))*XL**j/Dv).astype(float)*yv*np.sign(y) for j in FD]
        M = np.array(cols).T; sc = np.sqrt((M*M).sum(0)); sc[sc == 0] = 1
        d, *_ = np.linalg.lstsq(M/sc, r, rcond=1e-12); d = d/sc
        for j, v in zip(FN, d[:len(FN)]): N[j] += np.round(v)*np.spacing(abs(N[j]))
        for j, v in zip(FD, d[len(FN):]): D[j] += np.round(v)*np.spacing(abs(D[j]))
    e1 = np.mean(res(x, N, D, kn, kd) == b)
    print('%-22s exact %.4f -> %.4f  (head moves ulps N3..9 %s | D4..9 %s)' % (nm, e0, e1,
          [int(round((N[j]-N0[j])/np.spacing(abs(N0[j])))) for j in FN], [int(round((D[j]-D0[j])/np.spacing(abs(D0[j])))) for j in FD]), flush=True)
