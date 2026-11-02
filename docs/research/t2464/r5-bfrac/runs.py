"""T-3571 r5-bfrac: per-run decomposition on the t3571 probes. For each run (shape, z):
- ee: Excel's implied fraction error, cdf_e / (pdf_e * fl(x*y)) against the exact fraction r (stored lambda,
  y = fl(1-x)), units of 2^-53 relative; its mean and sd;
- ep: Excel's pdf error against the exact density (sd), and corr(ee, ep);
- the double TOMS fraction's own noise sd (model vs exact) and its corr with ee; terms = bfrac's iteration count.
Writes runs_<case>.tsv (derived cache, git-ignored) with per-point values for later scorers."""
import json, sys, math, mpmath, numpy as np
from pathlib import Path
from exact_r import r_exact
from loops import toms
HERE = Path(__file__).resolve().parent
CASES = HERE.parents[3] / 'eval/suites/probes-numeric/cases'
mpmath.mp.dps = 40
M = mpmath.mpf; U = M(2) ** -53

def lam_of(a, b, x, y):
    """Excel's bfrac lambda: fl(a - fl((a+b)x)) in both orientations (lamab.py; TOMS takes (a+b)y - b for a > b)."""
    return a - (a + b) * x

def load(case):
    meta = json.load(open(HERE / f'{case}.meta.json'))
    o = json.load(open(CASES / f'{case}.oracle.json'))['sheets']['Calc']
    cache = HERE / f'runs_{case}.tsv'
    old = {}
    if cache.exists():
        for l in cache.read_text().splitlines():
            k, rx, px = l.split('\t'); old[int(k)] = (M(rx), M(px))
    rows = []
    for k, a, b, z, x in meta:
        ca, cb = o.get(f'A{k}', {}), o.get(f'B{k}', {})
        if ca.get('t') != 'n' or cb.get('t') != 'n': continue
        y = 1 - x
        if k not in old:
            X, A, B = M(x), M(a), M(b)
            px = X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B)
            old[k] = (r_exact(a, b, x, y, lam_of(a, b, x, y)), px)
        rows.append(dict(k=k, a=a, b=b, z=z, x=x, y=y, cdf=ca['v'], pdf=cb['v'], rx=old[k][0], px=old[k][1]))
    cache.write_text(''.join(f'{k}\t{mpmath.nstr(v[0], 35)}\t{mpmath.nstr(v[1], 35)}\n' for k, v in sorted(old.items())))
    return rows

def groups(rows):
    g = {}
    for r in rows: g.setdefault((r['a'], r['b'], r['z']), []).append(r)
    return g

if __name__ == '__main__':
    for case in sys.argv[1:] or ['t3571-r5-bfrac-tune', 't3571-r5-bfrac-hold']:
        rows = load(case)
        print(case, len(rows))
        print('  (a, b) z      terms  ee mean  ee sd | pdf sd corr(ee,pdf) | TOMS-dbl noise sd corr(ee,TOMS)')
        for key, rs in groups(rows).items():
            ee = np.array([float((M(r['cdf']) / (M(r['pdf']) * M(r['x'] * r['y'])) / r['rx'] - 1) / U) for r in rs])
            ep = np.array([float((M(r['pdf']) / r['px'] - 1) / U) for r in rs])
            md, its = [], []
            for r in rs:
                v, it = toms(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y'])); its.append(it)
                md.append(float((M(v) / r['rx'] - 1) / U))
            md = np.array(md)
            print(f'  {key[0]:g},{key[1]:g} z{key[2]:+g}  {min(its):3d}  {ee.mean():+7.2f} {ee.std():6.2f} | {ep.std():7.2f} {np.corrcoef(ee, ep)[0,1]:+.2f} '
                  f'| {md.std():5.2f} {np.corrcoef(ee, md)[0,1]:+.2f}')
