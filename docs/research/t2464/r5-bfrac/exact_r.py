"""Exact bfrac fraction r (40 digits) at the double inputs a, b, x, y = fl(1-x) and the stored lambda = fl(a - fl((a+b)x)),
iterated to convergence. Caches to exact_r_<case>_<blk>.tsv (git-ignored derived data)."""
import sys, mpmath
from pathlib import Path
from data import load
mpmath.mp.dps = 40
def r_exact(a, b, x, y, lam):
    M = mpmath.mpf; a, b, x, y, lam = M(a), M(b), M(x), M(y), M(lam)
    c = 1 + lam; c0 = b / a; c1 = 1 + 1 / a; yp1 = y + 1; n = M(0); p = M(1); s = a + 1
    an, bn, anp1, bnp1 = M(0), M(1), M(1), c / c1; r = c1 / c
    for _ in range(2000):
        n += 1; t = n / a; w = n * (b - n) * x; e = a / s
        alpha = p * (p + c0) * e * e * (w * x)
        e = (1 + t) / (c1 + t + t); beta = n + w / s + e * (c + n * yp1); p = 1 + t; s += 2
        an, anp1 = anp1, alpha * an + beta * anp1
        bn, bnp1 = bnp1, alpha * bn + beta * bnp1
        r0 = r; r = anp1 / bnp1
        if abs(r - r0) < M(10) ** -35 * r: break
        an /= bnp1; bn /= bnp1; anp1 = r; bnp1 = M(1)
    return r
def get(case='t2464-w2f-xruns-1', blk=7):
    p = Path(__file__).with_name(f'exact_r_{case}_{blk}.tsv')
    rows = load(case, blk)
    if p.exists():
        v = [l.split('\t')[1] for l in p.read_text().splitlines()]
    else:
        v = []
        for r in rows:
            y = 1 - r['x']; lam = r['a'] - (r['a'] + r['b']) * r['x']
            v.append(mpmath.nstr(r_exact(r['a'], r['b'], r['x'], y, lam), 35))
        p.write_text(''.join(f"{r['x'].hex()}\t{s}\n" for r, s in zip(rows, v)))
    for r, s in zip(rows, v): r['rx'] = mpmath.mpf(s)
    return rows
if __name__ == '__main__':
    rows = get(*( (sys.argv[1], int(sys.argv[2])) if len(sys.argv) > 2 else ()))
    print(len(rows), rows[0]['rx'])
