import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from sc import load, res
from inj import res_inj, PH
exec(open(os.path.join(HERE, 'validate.py')).read().split('rows = []')[0].replace("MOVE = {('N', 5): 1234, ('D', 6): -777}", "MOVE = {('N', 5): int(os.environ.get('MV', 1234))}"))
for s in segs[:8]:
    xx = x[s]; tgt = res(xx, Nt, Dt); xm = xx[len(xx)//2]
    R = res_inj(xx, N0, D0, 0, 5, 1, PH); base = R[0]; u = np.spacing(np.abs(base))
    d = (tgt-base)/u; S = (R-base)/u
    match = [(r == tgt).mean() for r in R]
    v = stagevals(N0, xm)[('S', 5)][0]; uS = np.spacing(abs(v)); pred = (delta_pre('N', 5, 'S', xm)/uS) % 1
    print('x %.4f pred phi %.3f | best-match phi %.3f (match %.3f, at 0: %.3f) | nonzero resid %.2f' % (xm, pred, PH[int(np.argmax(match))], max(match), match[0], (d != 0).mean()))
