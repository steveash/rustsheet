import sys,math,mpmath
from rnd import *
L2=rn64(mpmath.log(2))
def LN(x): return rn53(rn64(L2*mpmath.log(mpmath.mpf(x),2)))
rows=[l.split() for l in open('gl_rows.txt') if 'r101' in l]
g=mpmath.euler
def score(f):
    return sum(f(float.fromhex(r[1]))==float.fromhex(r[2]) for r in rows)
D=lambda v: mpmath.mpf(v)
cands={
 '-LN(x)': lambda x: -LN(x),
 'fl(-LN + x*(-g)) dr': lambda x: rn53(rn64(D(-LN(x))+D(rn53(rn64(D(x)*D(float(-g))))))),
 'fl(-LN + x*(-g)) single': lambda x: float(D(-LN(x))+D(x*float(-g))),
 'exact -ln(x) - g x rounded': lambda x: rn53(-mpmath.log(D(x))-g*D(x)),
 'fl(-ln64 + x*-g)': lambda x: rn53(rn64(-rn64(L2*mpmath.log(D(x),2)) + D(x)*D(float(-g)))),
 'fl(x*-g - LN) via 64 ln, no store': lambda x: rn53(-rn64(L2*mpmath.log(D(x),2)) + D(rn53(D(x)*D(float(-g))))),
}
for k,f in cands.items(): print(f"{k:40s} {score(f)}/{len(rows)}")
