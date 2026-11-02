"""Score lentz.py variants per run of the t3571 probes (tune by default): exact count, corr with Excel's implied error."""
import sys, numpy as np, mpmath
from runs import load, groups, lam_of
import lentz as L
M = mpmath.mpf; U = M(2) ** -53

def score(rows, fn):
    ex, e, ee = 0, [], []
    for r in rows:
        v = fn(r)
        ex += (r['pdf'] * (r['x'] * r['y'])) * v == r['cdf']
        e.append(float((M(v) / r['rx'] - 1) / U))
        ee.append(float((M(r['cdf']) / (M(r['pdf']) * M(r['x'] * r['y'])) / r['rx'] - 1) / U))
    e, ee = np.array(e), np.array(ee)
    return ex, np.corrcoef(e, ee)[0, 1], np.std(ee - e)

VARIANTS = {
    'toms terms': lambda r: L.lentz(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']))[0],
    'CR terms': lambda r: L.lentz(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), terms=L.cr_terms)[0],
}
def _opt(**kw):
    tf = L.toms_terms_opt(**kw)
    return lambda r: L.lentz(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), terms=tf)[0]
for fx in (0, 1):
    for fb in (0, 1):
        for ext in (False, True):
            for wo in (0, 1):
                for bo in (0, 1):
                    VARIANTS[f'opt fx{fx} fb{fb} ext{int(ext)} w{wo} be{bo}'] = _opt(fx=fx, fb=fb, ext=ext, w_ord=wo, be_ord=bo)
if __name__ == '__main__':
    case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
    names = sys.argv[2].split(',') if len(sys.argv) > 2 else list(VARIANTS)
    g = groups(load(case))
    for nm in names:
        tot = 0; parts = []
        for key, rs in g.items():
            ex, c, sd = score(rs, VARIANTS[nm]); tot += ex
            parts.append(f'{key[0]:g},{key[1]:g},{key[2]:g}:{ex}/{len(rs)} {c:+.2f}')
        print(f'{nm}: total {tot}' + ('\n   ' + '\n   '.join(parts) if '-v' in sys.argv else ''))
