"""Diagnostic on the current best model (Lentz, explicit terms, eps 10*DBL_EPSILON): replace all alpha_n, all beta_n,
or b0 by correctly rounded values; per-run exact counts. A rise means Excel's rounding of that term is not modelled."""
import sys
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
alf = sys.argv[2] if len(sys.argv) > 2 else '(u/s)*(v/s)*(w*x)'
def mixed(sw):
    def mk(a, b, x, y, lam):
        b0, g1 = L.explicit_terms(alf, 'n + w/s + e2*X')(a, b, x, y, lam); b0c, g2 = L.cr_terms(a, b, x, y, lam)
        def gen():
            for (a1, be1), (a2, be2) in zip(g1, g2):
                yield (a2 if 'a' in sw else a1), (be2 if 'b' in sw else be1)
        return (b0c if '0' in sw else b0), gen()
    return mk
g = groups(load(case))
for sw in ('', 'a', 'b', '0', 'ab'):
    tf = mixed(sw)
    per = [sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz2(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), tf)[0] == r['cdf'] for r in rs) for rs in g.values()]
    print(f'CR {sw or "none":5s}: {sum(per)} {per}')
