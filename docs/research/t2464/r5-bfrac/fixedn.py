"""Exact counts per run with the Lentz loop forced to N fraction terms (no stop test), explicit terms; and the
|C*D - 1| at the last term: shows the stop rule Excel uses."""
import sys, numpy as np
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
tf = L.explicit_terms('(u/s)*(v/s)*(w*x)', 'n + w/s + e2*X')
def run_n(r, N):
    b0, T = tf(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']))
    f = 1e-30; C = f; D = 0.0
    D = 1.0 / (b0 + D); C = b0 + 1.0 / C; f = f * (C * D)
    des = []
    for k, (al, be) in zip(range(N), T):
        D = 1.0 / (be + al * D); C = be + al / C; de = C * D; f = f * de; des.append(abs(de - 1.0))
    return f, des
for key, rs in groups(load(case)).items():
    out = []
    for N in range(3, 32):
        ex = 0; dl = []
        for r in rs:
            f, des = run_n(r, N); ex += (r['pdf'] * (r['x'] * r['y'])) * f == r['cdf']; dl.append(des[-1])
        if ex: out.append(f'N{N}:{ex}(|d-1|~{np.median(dl):.1e})')
    print(key, ' '.join(out))
