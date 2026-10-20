# K2 "hidden c/x at the hand-off" model (reviewer 3, 2026-10-20). grid_a's stored x87 Horner, except that the hand-off
# sum carries +c/x (formed in extended precision, one store) and the next constant is reduced by c. The polynomial is
# algebraically unchanged (c/x*x - c = 0); only the hand-off rounding sees the term. Usage: python cx.py r112 r901 ...
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p, q, sim
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def chainc(c,x,K,cc):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        P=sd(s*x)
        if i==K: s=sd(P+LD(c[i])+LD(cc)/x)
        elif i==K+1: s=sd(P+LD(c[i]-cc))
        else: s=sd(P+LD(c[i]))
    return s
u=2.0**-54; CN=-3128*u; CD=552*u
def model(X0):
    X=X0.astype(LD); return sd(sd(X-2)*sd(chainc(p,X,4,CN)/chainc(q,X,3,CD))).astype(np.float64)
if __name__=='__main__':
    for case in sys.argv[1:]:
        X0,Y=load(case); print(case, len(X0), 'grid_a', round(np.mean(sim(X0)==Y),3), 'c/x model', round(np.mean(model(X0)==Y),3))
