# sample points inside LP polytope (d2 fixed) via hit-and-run; print d vectors
import sys, numpy as np
fn=sys.argv[1]; deg=int(sys.argv[2]); R=float(sys.argv[3]); d2=int(sys.argv[4]); ns=int(sys.argv[5]); marg=float(sys.argv[6]); start=[float(v) for v in sys.argv[7].split(',')]
Z=[];LO=[];HI=[]
for l in open(fn):
    a=l.split()
    if a[0]!='C': continue
    lo=float(a[2]); hi=float(a[3])
    if lo>-R or hi<R: Z.append(float.fromhex(a[1])); LO.append(lo-d2+marg); HI.append(hi-d2-marg)
Z=np.array(Z); LO=np.array(LO); HI=np.array(HI)
A=np.vstack([(Z/0.25)**j for j in range(1,deg+1)]).T
x=np.array(start); v=A@x; assert ((v>=LO)&(v<=HI)).all(), ((v<LO)|(v>HI)).sum()
rng=np.random.default_rng(1)
for it in range(ns):
    d=rng.normal(size=deg); d/=np.linalg.norm(d)
    ad=A@d; ax=A@x
    with np.errstate(divide='ignore',invalid='ignore'):
        t1=(LO-ax)/ad; t2=(HI-ax)/ad
    tl=np.where(ad>0,t1,t2); th=np.where(ad>0,t2,t1)
    tl=tl[np.isfinite(tl)].max(); th=th[np.isfinite(th)].min()
    if th<tl: continue
    x=x+d*rng.uniform(tl,th)
    print(d2,' '.join('%.6f'%u for u in x))
