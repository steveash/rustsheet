"""EXP internals variants applied to the extended-E leader (cscan.prep) per sheet."""
import sys, collections, mpmath
from lib import load_probe, D, R64, R53, mpf, LOG2E64
from cscan import prep
def EXP(E, v):
    E=mpf(E)
    t = R64(E*LOG2E64)
    if 'tD' in v: t = mpf(D(E*LOG2E64))
    if 'tL2E53' in v: t = R64(E*mpf(float(1/mpmath.ln(2))))
    n = mpmath.nint(t); f = t-n
    if 'nofold' in v:
        u = R64(mpf(2)**f - 1)
        if 'uD' in v: u=mpf(D(u))
        w = R64(1+u)
        if 'wD' in v: w=mpf(D(w))
        return R53(w*mpf(2)**n)
    u = R64(mpf(2)**abs(f)-1)
    if 'uD' in v: u=mpf(D(u))
    w = R64(1+u)
    if 'wD' in v: w=mpf(D(w))
    if f<0:
        w = R64(1/w)
        if 'rD' in v: w=mpf(D(w))
    return R53(w*mpf(2)**n)
VARS=['v6','tD','tL2E53','uD','wD','rD','wD rD','nofold','nofold uD','nofold wD','tD wD']
for sh in sys.argv[1:]:
    rows=[r for r in load_probe('probe1') if r['case']==sh]
    pre=prep(rows)
    c=collections.Counter()
    for E,v,zn,r in pre:
        for var in VARS: c[var]+= EXP(E,var)==v
    print(sh, rows[0]['a'], rows[0]['b'], {k:c[k] for k in VARS})
