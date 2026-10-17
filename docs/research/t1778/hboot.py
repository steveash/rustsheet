# T-1778: bootstrap the coarse polynomial H(z) from hi step positions (run in the extract_runs.py OUTDIR with covm built there).
# Usage: BD=1 ZR=1e-3 python3 hboot.py DEG ZMAX
import math, subprocess, sys
import numpy as np
from scipy.optimize import linprog
A=1.1170334248975802; U=math.ulp(A)
deg=int(sys.argv[1]) if len(sys.argv)>1 else 3
cons=[]
Z={}
for l in open('zs3.txt'):
    p=l.split(); r=int(p[0])
    if r not in Z: x=float.fromhex(p[1]); Z[r]=x*x
for r in range(24): cons.append((Z[r], 6335723 if r<=10 else 6335724))
for r in range(24): cons.append((Z[24+r], 63357233 if r<=20 else 63357234))
for r in range(24): cons.append((Z[48+r], 633572509 if r<=12 else 633572510))
Ze={}
for l in open('runs.txt'):
    p=l.split(); r=int(p[0])
    if r not in Ze: x=float.fromhex(p[1]); Ze[r]=x*x
for r in range(1,62): cons.append((Ze[r], 1 if r<=14 else 2))
for r in range(64,112):
    if r in Ze: cons.append((Ze[r], 634 if r<=88 else 635))
for r in range(112,160):
    if r in Ze and r!=149: cons.append((Ze[r], 126714 if r<=113 else (126715 if r<=139 else 126716)))
ZR=float(__import__('os').environ.get('ZR','1e-3'))
sc=[ZR**k for k in range(deg+1)]
nom=np.array([0,0.1406813126414/U*sc[1],0.0448192/U*sc[2] if deg>=2 else 0,0.0018/U*sc[3] if deg>=3 else 0]+[0]*(deg-3))[:deg+1]
def rows(z): return [z**k/sc[k] for k in range(deg+1)]
def lp(c_obj,extra_bound=None):
    Aub=[];bub=[]
    for z,m in cons:
        r=np.array(rows(z)); b0=r@nom
        Aub.append(r); bub.append(m+0.5-b0)
        Aub.append(-r); bub.append(-(m-0.5-b0))
    BD=float(__import__('os').environ.get('BD','1'))
    bnds=[(None,None)]*3+[((-BD/U*sc[k]-nom[k]),(BD/U*sc[k]-nom[k])) for k in range(3,deg+1)]
    res=linprog(c_obj,A_ub=np.array(Aub),b_ub=np.array(bub),bounds=bnds,method='highs')
    return res
def Hrange(z):
    r=np.array(rows(z)); b0=r@nom
    lo=lp(r); hi=lp(-r)
    if lo.status or hi.status: return None
    return lo.fun+b0, -hi.fun+b0
# runs to bootstrap from every2.txt sorted by z
runs={}
for l in open('runs.txt'):
    p=l.split(); r=int(p[0]); runs.setdefault(r,[]).append(float.fromhex(p[1]))
order=sorted([(min(x*x for x in v),r) for r,v in runs.items() if 5e-7<min(x*x for x in v)<float(sys.argv[2]) and r!=217 and r not in (218,219,220)])
for z,r in order:
    xs=runs[r]; zmin=min(x*x for x in xs); zmax=max(x*x for x in xs)
    rg=Hrange(zmin)
    if rg is None: print('infeasible at run',r); cons[:]=cons[:-2] if lastadd else cons; break
    lo,hi=math.floor(rg[0]-0.5),math.ceil(rg[1]+0.5)
    if hi-lo>6000: print('run',r,'range too wide',hi-lo); continue
    ms=list(range(lo,hi+1))
    out=subprocess.run(['./covm','runs.txt',str(r),'1']+[str(m) for m in ms],capture_output=True,text=True).stdout.split()
    counts=[int(t.split(':')[1]) for t in out]
    n=len(xs); mx=max(counts); good=[ms[i] for i,c in enumerate(counts) if c==mx and c>=n-2]
    print('run %d z %.4g..%.4g n %d range %.2f..%.2f  good %s'%(r,zmin,zmax,n,rg[0],rg[1],good[:10]))
    lastadd=False
    if len(good)==1 and zmax/zmin<1+1e-9:
        cons.append((zmin,good[0])); cons.append((zmax,good[0])); lastadd=True
        if Hrange(zmin) is None: print('  -> adding run',r,'makes infeasible; dropping'); cons[:]=cons[:-2]
res=lp([0]*(deg+1))
# report center via margin LP not needed; print ranges
for k in range(deg+1):
    e=[0]*(deg+1); e[k]=1
    a=lp(e); b=lp([-v for v in e])
    print('param',k,'[',(a.fun+nom[k])/sc[k]*(U if k else 1),',',(-b.fun+nom[k])/sc[k]*(U if k else 1),']')
