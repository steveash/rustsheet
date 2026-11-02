"""Enumerate alpha_n forms (argv[2]: factor sets, e.g. "u,v,w,x;u,v,m,q"; q = x*x, k = b-n, W = n*x*(b-n)): left-associated sequences over the factors in every order, with the two
divisions by s at every pair of slots (also 's*s' as one divisor), and the factors (u, v, m, x, x); plus the pair
products (u/s)*(v/s)*(w*x)-style forms. Lentz loop, beta and b0 as modelled. Prints the top forms by tune exact."""
import itertools, sys
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
g = list(groups(load(case)).values())
BETA = sys.argv[3] if len(sys.argv) > 3 else 'R: n + w/s + e2*X'

def seqform(order, divpos, ss):
    """order: factor names; divpos: slots (0..len) after which '/s' is applied (ss: one '/(s*s)' at divpos[0])."""
    def f(u, v, s, m, w, x, a, b, n):
        P = (n - 1.0) / a + 1.0; Q = P + b / a; E = a / s
        val = dict(u=u, v=v, w=w, x=x, m=m, q=x * x, n=n, k=b - n, W=n * x * (b - n), i=1.0 / s, P=P, Q=Q, E=E, R=u / a, S=v / a)
        acc = None
        for i, nm in enumerate(order):
            acc = val[nm] if acc is None else acc * val[nm]
            if ss and i + 1 == divpos[0]: acc = acc / (s * s)
            if not ss:
                for d in divpos:
                    if d == i + 1: acc = acc / s
        return acc
    return f
FACS = [tuple(f.split(',')) for f in (sys.argv[2] if len(sys.argv) > 2 else 'u,v,w,x;u,v,m,x,x').split(';')]
forms = {}
for facs in FACS:
    for order in set(itertools.permutations(facs)):
        k = len(order)
        if 's' in facs[-1:]: continue
        if 'E' in facs: forms[' '.join(order)] = seqform(order, (), False); continue
        for d in itertools.combinations_with_replacement(range(1, k + 1), 2):
            forms[' '.join(order) + f' /s@{d}'] = seqform(order, d, False)
        for d in range(1, k + 1):
            forms[' '.join(order) + f' /ss@{d}'] = seqform(order, (d,), True)
pairs = {'u/s': lambda u, v, s, w, x: u / s, 'v/s': lambda u, v, s, w, x: v / s, 'wx': lambda u, v, s, w, x: w * x,
         'uv': lambda u, v, s, w, x: u * v, 'ss': lambda u, v, s, w, x: s * s}
print(len(forms), 'forms', file=sys.stderr)
res = []
for nm, f in forms.items():
    L.ALPHA_FORMS['_t'] = f
    tf = L.explicit_terms('_t', BETA)
    per = [sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz2(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), tf)[0] == r['cdf'] for r in rs) for rs in g]
    res.append((sum(per), nm, per))
for t in sorted(res, reverse=True)[:15]: print(t)
