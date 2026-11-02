"""Per-index diagnostic on the current model: replace alpha_n (or beta_n) at one n by its correctly rounded value;
exact count per run. Rows: runs with >= 14 terms."""
import sys
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
alf = 'v/s/s*u*m*x*x'
def mixed(kind, idx):
    def mk(a, b, x, y, lam):
        b0, g1 = L.explicit_terms(alf, 'n + w/s + e2*X')(a, b, x, y, lam); _, g2 = L.cr_terms(a, b, x, y, lam)
        def gen():
            for n, ((a1, be1), (a2, be2)) in enumerate(zip(g1, g2), 1):
                yield (a2 if kind == 'a' and n == idx else a1), (be2 if kind == 'b' and n == idx else be1)
        return b0, gen()
    return mk
for key, rs in groups(load(case)).items():
    if key[2] != -1.0: continue
    def ex(tf): return sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz2(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), tf)[0] == r['cdf'] for r in rs)
    base = ex(L.explicit_terms(alf, 'n + w/s + e2*X'))
    print(key, 'base', base, 'alpha:', ' '.join(f'{n}:{ex(mixed("a", n)) - base:+d}' for n in range(1, 24)))
    print(' ' * len(str(key)), '      beta: ', ' '.join(f'{n}:{ex(mixed("b", n)) - base:+d}' for n in range(1, 24)))
