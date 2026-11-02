"""For model misses on the long runs, nudge one term (alpha_n or beta_n, +-1 ulp) at a time and count which nudges
make the cell bit-exact; tallies by term kind and by n relative to the last term."""
import sys, math, collections
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
alf = sys.argv[2] if len(sys.argv) > 2 else 'v/s/s*u*m*x*x'
base = L.explicit_terms(alf, 'n + w/s + e2*X')
def nudged(kind, idx, dirn):
    def mk(a, b, x, y, lam):
        b0, g = base(a, b, x, y, lam)
        def gen():
            for n, (al, be) in enumerate(g, 1):
                if n == idx and kind == 'a': al = math.nextafter(al, dirn * math.inf)
                if n == idx and kind == 'b': be = math.nextafter(be, dirn * math.inf)
                yield al, be
        return b0, gen()
    return mk
tal = collections.Counter(); tn = collections.Counter(); miss = 0; fixed = 0
for key, rs in groups(load(case)).items():
    if key[2] == -7.0: continue
    for r in rs:
        args = (r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']))
        f, N = L.lentz2(*args, base)
        if (r['pdf'] * (r['x'] * r['y'])) * f == r['cdf']: continue
        miss += 1; hit = []
        for kind in 'ab':
            for n in range(1, N + 1):
                for d in (1, -1):
                    if (r['pdf'] * (r['x'] * r['y'])) * L.lentz2(*args, nudged(kind, n, d))[0] == r['cdf']: hit.append((kind, n, N))
        if hit: fixed += 1
        for kind, n, N in hit: tal[kind] += 1 / len(hit); tn[(kind, min(n, 6), 'first' if n <= 3 else 'mid' if n < N - 2 else 'last3')] += 1 / len(hit)
print('misses', miss, 'fixable by one nudge', fixed)
print('by kind', {k: round(v, 1) for k, v in tal.items()})
for k in sorted(tn): print(k, round(tn[k], 1))
