# Near-2 ladders with grid_a stored (99.5%): what are the 0.5% mismatches? residue dependence at P4/s4 and stage attribution.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(np.asarray(v,dtype=np.float64)); return (np.nextafter(a,np.inf)-a)
u=2.0**-54
Xs=[];Ys=[]
for c in ['e501','e502','e503','e504','e511','e512','r204','e401','e402','e403']:
    X,Y=load(c); m=(X>2)&(X<=3); Xs.append(X[m]); Ys.append(Y[m])
X=np.concatenate(Xs); YC=np.concatenate(Ys); XC=X.astype(LD); TC=sd(XC-LD(2)); UY=ulp(YC)
def chain(c,x,unst=set()):
    s=LD(c[0])+0*x; out=[]
    for i in range(1,len(c)):
        pe=s*x; p=sd(pe); se=p+LD(c[i]); s=se if i in unst else sd(se); out.append((pe,p,se,s))
    return out
for label,P,Q,unN,unD in (('grid_a stored',list(P0),list(Q0),set(),set()),('grid_a M*',list(P0),list(Q0),{3},{2})):
    cN=chain(P,XC,unN); cD=chain(Q,XC,unD); N=cN[-1][3]; D=cD[-1][3]; qd=sd(N/D); y=sd(TC*qd)
    d=np.array((YC.astype(LD)-y)/UY,dtype=float); mis=d!=0
    print(f"{label}: n={len(X)} mismatch {mis.mean():.4f}; Excel-sim ulp: "+", ".join(f"{int(k)}:{int(v)}" for k,v in zip(*np.unique(d[mis],return_counts=True))))
    pe4,p4,se4,s4=cN[3]; r4=np.array((pe4-p4)/ulp(p4).astype(LD),dtype=float); f4=np.array((se4/ulp(s4).astype(LD))%1,dtype=float)
    bins=np.linspace(-0.5,0.5,11); print("   mismatch by P4 residue: "+" ".join(f"{np.mean(mis[(r4>=lo)&(r4<hi)]):.4f}" for lo,hi in zip(bins,bins[1:])))
    vals,cnt=np.unique(np.round(f4*16)/16,return_counts=True); print("   s4 pre-rounding positions (mod ulp s4) present: "+" ".join(f"{v:.3f}({c})" for v,c in zip(vals,cnt) if c>100))
    print("   mismatch by s4 position: "+" ".join(f"{v:.3f}:{np.mean(mis[np.isclose(f4,v,atol=0.02)]):.4f}" for v,c in zip(vals,cnt) if c>100))
    for k in range(1,9):
        pe,p,se,s=cN[k]; pos=np.array((se/ulp(s).astype(LD))%1,dtype=float); uniq=len(np.unique(np.round(pos*64)))
        print(f"   N.s{k+1}: distinct positions(/64) {uniq:3d}; mean pos {pos.mean():.3f}; mismatch at pos≈0.5: {np.mean(mis[np.abs(pos-0.5)<0.02]) if np.any(np.abs(pos-0.5)<0.02) else float('nan'):.4f} (n={np.sum(np.abs(pos-0.5)<0.02)})")
    rq=np.array((N/D-qd)/ulp(qd).astype(LD),dtype=float); print("   mismatch by quotient residue: "+" ".join(f"{np.mean(mis[(rq>=lo)&(rq<hi)]):.4f}" for lo,hi in zip(bins,bins[1:])))
