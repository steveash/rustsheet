import os, sys, numpy as np
sys.path.insert(0, '/home/user/rustsheet/docs/research/t1941/review3'); os.chdir('/home/user/rustsheet')
from spec import load, p as P0, q as Q0
LD = np.longdouble
def sd(v): return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)
RUNS = 'r701 r112 p401 r702 r610 r901 r703 r113 r704 r611 r705 v401a v404a v407a'.split(); NPT = int(os.environ.get('NPT', 4096))
DATA = []
for case in RUNS:
    X0, Y0 = load(case)
    br = [0] + [i + 1 for i in range(len(X0) - 1) if np.nextafter(X0[i], np.inf) != X0[i + 1]] + [len(X0)]
    a, b = max(zip(br, br[1:]), key=lambda t: t[1] - t[0]); DATA.append((X0[a:b][:NPT].astype(LD), Y0[a:b][:NPT]))
