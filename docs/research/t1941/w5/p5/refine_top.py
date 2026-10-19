"""Local refinement of a top-stage scan peak (a mod 1, b, c) on the exact S-site objective (T-1941 W5 part 5).
usage: refine_top.py control|real a b c   env: BASE, CHAIN, J (as scan2.py)"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
mode = sys.argv[1]; a0, b0, c0 = map(float, sys.argv[2:5])
src = open(os.path.join(HERE, 'scan2.py')).read()
src = src.split('# null for the joint objective')[0]          # data, curves, truth, obj, joint
sys.argv = [sys.argv[0], mode]
exec(src)
def f(p): return float(obj(CS, ((p[0]+p[1]*dx+p[2]*dx**2)*u0)/uS))
best = np.array([a0, b0, c0]); bv = f(best)
for step in [(0.02, 0.2, 0.2), (0.01, 0.05, 0.05), (0.004, 0.02, 0.02), (0.002, 0.005, 0.005)]:
    improved = True
    while improved:
        improved = False
        for d in np.array([(i, j, k) for i in (-1, 0, 1) for j in (-1, 0, 1) for k in (-1, 0, 1) if (i, j, k) != (0, 0, 0)])*np.array(step):
            v = f(best+d)
            if v > bv: bv = v; best = best+d; improved = True
print('refined a %.4f b %.4f c %.4f  S objective %.1f' % (best[0] % 1, best[1], best[2], bv))
if truth: print('truth   a %.4f b %.4f c %.4f  S objective %.1f' % (truth[0] % 1, truth[1], truth[2], f(np.array(truth))))
