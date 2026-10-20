# Near-2 and far mismatch structure by stage ties / product residues; p4 parity variants with the near-2 constraint kept via p1.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(np.asarray(v,dtype=np.float64)); return (np.nextafter(a,np.inf)-a)
u=2.0**-54
PQ=np.load('SCRATCH/cand9_PQ.npy'); P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
Xs=[];Ys=[]
for c in ['e501','e502','e503','e504','e511','e512','r204','e401','e402','e403']:
    X,Y=load(c); m=(X>2)&(X<=3); Xs.append(X[m]); Ys.append(Y[m])
XN=np.concatenate(Xs); YN=np.concatenate(Ys)
clus=[]
for nn in range(1,17):
    for h in 'ab':
        X0,Y0=load('q2%02d%s'%(nn,h)); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
        clus+=[(X0[a:b],Y0[a:b]) for a,b in zip(br,br[1:]) if b-a>=256]
XF=np.concatenate([Xc for Xc,Yc in clus]); YF=np.concatenate([Yc for Xc,Yc in clus])
def chain(c,x):
    s=LD(c[0])+0*x; out=[]
    for i in range(1,len(c)):
        pe=s*x; p=sd(pe); se=p+LD(c[i]); s=sd(se); out.append((pe,p,se,s))
    return out
def analyse(label,P,Q,X,Y):
    XC=X.astype(LD); TC=sd(XC-LD(2)); UY=ulp(Y); cN=chain(P,XC); cD=chain(Q,XC); y=sd(TC*sd(cN[-1][3]/cD[-1][3]))
    d=np.array((Y.astype(LD)-y)/UY,dtype=float); mis=d!=0; print(f"{label}: n={len(X)} mismatch {mis.mean():.4f}")
    for k in (2,3,4,5):
        pe,p,se,s=cN[k-1]; pos=np.array((se/ulp(s).astype(LD))%1,dtype=float); r=np.array((pe-p)/ulp(p).astype(LD),dtype=float); tie=np.abs(pos-0.5)<0.01
        print(f"   N.s{k}: ties {np.mean(mis[tie]) if tie.any() else float('nan'):.4f} (n={tie.sum()}) non-ties {np.mean(mis[~tie]):.4f}; by P{k} residue: "+" ".join(f"{np.mean(mis[(r>=lo)&(r<lo+0.2)]):.4f}" for lo in (-0.5,-0.3,-0.1,0.1,0.3)))
analyse("cand9 NEAR-2",P,Q,XN,YN); analyse("grid_a NEAR-2",list(P0),list(Q0),XN,YN); analyse("cand9 FAR",P,Q,XF,YF)
g1=ulp(P0[1])/u
for d4,d5 in ((-5209,3128),(-5208,3126),(-5210,3130)):
    P2=list(P); P2[4]=float(P0[4]+d4*u); P2[5]=float(P0[5]+d5*u)
    d3=(P[3]-P0[3])/u; d2=(P[2]-P0[2])/u; d1=(-(2*d4+d5)-4*d3-8*d2)/16; P2[1]=float(P0[1]+np.round(d1/g1)*g1*u)
    for nm,X,Y in (("near",XN,YN),("far",XF,YF)):
        XC=X.astype(LD); yy=sd(sd(XC-LD(2))*sd(chain(P2,XC)[-1][3]/chain(Q,XC)[-1][3])).astype(np.float64); print(f"   dp4={d4} dp5={d5} {nm} {np.mean(yy==Y):.4f}")
