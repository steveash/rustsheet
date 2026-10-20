# (a) hand-off rule (f, parity) at s4 and (g, parity) at d3 for EVERY run >= 1000 consecutive doubles in (2,3]: delta mod 2 ulp per run.
# (b) within-run drift: 8 chunks per long run, rule per chunk.
import sys, glob, os, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def rn(v,g,odd):
    q=v/g; f=np.floor(q); r=q-f; fo=np.mod(f,2)==1
    up=(r>0.5)|((r==0.5)&(fo if not odd else ~fo)); return (f+up)*g
F=np.arange(-4,4)/8.0
def prep(X):
    s=LD(P0[0])+0*X
    for i in range(1,4): s=sd(sd(s*X)+LD(P0[i]))
    v4=sd(s*X)+LD(P0[4]); s4=sd(v4); d=LD(Q0[0])+0*X
    for i in range(1,3): d=sd(sd(d*X)+LD(Q0[i]))
    v3=sd(d*X)+LD(Q0[3]); d3=sd(v3); return v4,s4,ulp(s4),v3,d3,ulp(d3)
def fitrule(X,Y):
    t=sd(X-LD(2)); v4,s4,u,v3,d3,uD=prep(X)
    def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
    base=np.mean(yof(s4,d3)==Y)
    rN={(f,o):np.mean(yof(rn(v4+f*u,u,o),d3)==Y) for f in F for o in (False,True)}; kN=max(rN,key=rN.get); s4b=rn(v4+kN[0]*u,u,kN[1])
    rD={(g,o):np.mean(yof(s4b,rn(v3+g*uD,uD,o))==Y) for g in F for o in (False,True)}; kD=max(rD,key=rD.get)
    return base,kN,rN[kN],kD,rD[kD]
cases=sorted(set(os.path.basename(p).replace('kmyst-gam-','').replace('.case.json','') for p in glob.glob('eval/suites/probes-numeric/cases/kmyst-gam-*.case.json')))
for case in cases:
    try: X0,Y0=load(case)
    except Exception: continue
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    for a,b in zip(br,br[1:]):
        if b-a<1000: continue
        X=X0[a:b].astype(LD); Y=Y0[a:b]; x=float(X[(b-a)//2])
        if not (2.0<x<=3.0): continue
        base,kN,rNb,kD,rDb=fitrule(X,Y)
        print(f"   {case:7s} x={x:.5f} n={b-a:5d} | N: f={kN[0]:+.3f} {'odd' if kN[1] else 'even'} {base:.3f}->{rNb:.3f} | D: g={kD[0]:+.3f} {'odd' if kD[1] else 'even'} ->{rDb:.3f}", flush=True)
