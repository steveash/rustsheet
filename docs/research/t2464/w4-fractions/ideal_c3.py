"""Basis of idealized tie responses, then search 0/1 combinations against Excel's c (a,b>1 blocks)."""
from ideal_c import *
import itertools
rows=[]
for a,b,x,c in EXCEL[:6]:
    a,b,x=M(a),M(b),M(x); y=1-x; h=M(10)**-20
    def dl(f, v): return float((mp.log(f(v*(1+h)))-mp.log(f(v)))/h)
    p = x**(a-1)*y**(b-1)/mp.beta(a,b); Ix=I(a,b,x)
    K={}
    K['b']=float(b)
    K['-ay/x']=float(-a*y/x)
    K['cmp']=float(-y*p/Ix)
    # CF incbcf in x, with x := 1-y
    K['cfx(1-y)']= dl(lambda yy: incbcf(a,b,1-yy), y)
    K['incbd z']= dl(lambda yy: incbd(a,b,x/yy), y)
    K['-1']=-1.0
    K['bf yp1']= dl(lambda yy: bfrac(a,b,x,yy,a-(a+b)*x), y)
    K['bf lam']= dl(lambda yy: bfrac(a,b,x,y,(a+b)*yy-b), y)
    rows.append((float(a),float(b),float(x),c,K))
names=list(rows[0][4])
for r in rows: print(r[:4], ' '.join(f"{k}={v:+.2f}" for k,v in r[4].items()))
best=[]
for combo in itertools.product([0,1],repeat=len(names)):
    pred=[sum(n*r[4][k] for n,k in zip(combo,names)) for r in rows]
    err=sum((p-r[3])**2 for p,r in zip(pred,rows))
    best.append((err,combo,pred))
best.sort()
for e,cmb,pred in best[:8]:
    print(f"{e:8.2f}", [k for n,k in zip(cmb,names) if n], ' '.join(f"{p:+.2f}" for p in pred))
