"""Where the top-stage information lives: per-x-bin truth contribution vs null spread, from a PERRUN dump of
scan2.py/scan3.py (T-1941 W5 part 5). usage: perrun.py FILE.npz"""
import numpy as np, sys
z = np.load(sys.argv[1]); xm, tv, sd = z['xm'], z['tv'], z['sd']
print('%s: total truth %.1f, null sd %.1f (Z %.1f)' % (sys.argv[1].split('/')[-1], tv.sum(), np.sqrt((sd**2).sum()), tv.sum()/np.sqrt((sd**2).sum())))
for lo in np.arange(0.7, 1.5, 0.1):
    m = (xm >= lo) & (xm < lo+0.1)
    if m.sum():
        s = np.sqrt((sd[m]**2).sum()); print('  x %.1f-%.1f runs %3d  truth %7.1f  null sd %6.1f  Z %5.1f  Z^2/run %.3f' % (lo, lo+.1, m.sum(), tv[m].sum(), s, tv[m].sum()/s, (tv[m].sum()/s)**2/m.sum()))
