import sys,mpmath
from rnd import *
D=mpmath.mpf
L2=rn64(mpmath.log(2))
def st(v): return rn53(rn64(v))
def LN(x): return st(L2*mpmath.log(D(x),2))
B=[mpmath.bernoulli(2*k)/(2*k*(2*k-1)) for k in range(1,14)]
C=[float(b) for b in B]
def gl(y,nt):
    z=st(1/D(y)); z2=st(D(z)*D(z))
    r=C[nt-1]
    for k in range(nt-2,-1,-1): r=st(D(st(D(r)*D(z2)))+D(C[k]))
    s=st(D(r)*D(z))
    a=st(D(y)-D(0.5)); b=st(D(a)*D(LN(y))); d=st(D(b)-D(y)); e=st(D(d)+D(0.9189385332046727)); return st(D(e)+D(s))
want=sys.argv[1:]
rows=[l.split() for l in open('runs.txt')]
for nm in want:
    rr=[r for r in rows if r[4]==nm and r[1]!='nan']
    print(nm, len(rr), {nt:sum(gl(float.fromhex(r[0]),nt)==float.fromhex(r[1]) for r in rr[::4]) for nt in (9,10,12)}, '(every 4th row)')
