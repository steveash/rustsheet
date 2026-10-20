"""K1 fingerprint repeatability: 3 disjoint-seed subsamples; calibration with AN2 and with lat1 (another K1 set)."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..', 'p5'))
from fp import L, st, fit, stats
from k1fp_lib import ldND, G
z = np.load(sys.argv[1])
sets = {'AN2': os.path.join(HERE, '..', '..', 'g_candidate_w4_AN2.txt'), 'lat1': os.path.join(HERE, '..', '..', 'w4', 'g_candidate_lat1.txt')}
for seed in (11, 12, 13):
    i = np.random.default_rng(seed).choice(len(z['x']), 200000, replace=False); x, y = z['x'][i], z['b'][i]
    out = ['real %.3f' % stats(fit(x, L(y), 7, 7, 1.0, 2.0)[2])[0]]
    for nm, p in sets.items():
        N, D = ldND(p)
        for kn, kd, lab in ((99, 99, 'st'), (2, 3, 'W4top'), (3, 4, 'top4/3')):
            out.append('%s/%s %.3f' % (nm, lab, stats(fit(x, L(G(x, N, D, kn, kd)), 7, 7, 1.0, 2.0)[2])[0]))
    print('seed %d: ' % seed + '  '.join(out), flush=True)
