"""Diagnostic: Lentz on TOMS's double terms, with one term (alpha_n or beta_n) at a time replaced by its correctly
rounded value; exact counts per run show which TOMS term roundings Excel does not share."""
import sys
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
zs = [float(v) for v in (sys.argv[2] if len(sys.argv) > 2 else '-7').split(',')]
def mixed(swap):
    def mk(a, b, x, y, lam):
        b0, g1 = L.toms_terms(a, b, x, y, lam); b0c, g2 = L.cr_terms(a, b, x, y, lam)
        def gen():
            for n, ((a1, be1), (a2, be2)) in enumerate(zip(g1, g2), 1):
                yield (a2 if ('a', n) in swap or 'all' in swap else a1), (be2 if ('b', n) in swap or 'all' in swap else be1)
        return (b0c if 'b0' in swap or 'all' in swap else b0), gen()
    return mk
for key, rs in groups(load(case)).items():
    if key[2] not in zs: continue
    def ex(sw):
        tf = mixed(sw)
        return sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), terms=tf)[0] == r['cdf'] for r in rs)
    base = ex(set()); out = [f'none:{base}', f"all:{ex({'all'})}", f"b0:{ex({'b0'})}"]
    for n in range(1, 9):
        out.append(f'a{n}:{ex({("a", n)})} b{n}:{ex({("b", n)})}')
    print(key, ' '.join(out))
