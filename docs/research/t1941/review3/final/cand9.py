# Constrained scan under the STORED structure: dp3 on its grid; dp2 from the bias-curve LS; dp1 from the exact near-2 constraint
# 16 dp1 + 8 dp2 + 4 dp3 = 7290 u  (dp4 = -5209, dp5 = +3128 fixed). Needs allXY.npy (from allscore.py) and cand7_PQ.npy (any reference set).
import sys, numpy as np, time
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(np.asarray(v,dtype=np.float64)); return (np.nextafter(a,np.inf)-a)
u=2.0**-54
X,Y=np.load('SCRATCH/allXY.npy')
rng=np.random.default_rng(1); near=X<2.02; far=np.where(~near)[0]; pick=rng.choice(far,400000,replace=False)
sel=np.concatenate([np.where(near)[0],pick]); XS=X[sel]; YS=Y[sel]; nearS=XS<2.02; XSl=XS.astype(LD); TS=sd(XSl-LD(2))
def hm(c,x):
    s=LD(c[0])+0*x
    for i in range(1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def sc(P,Q):
    e=sd(TS*sd(hm(P,XSl)/hm(Q,XSl))).astype(np.float64)==YS; return float(e.mean()),float(e[nearS].mean()),float(e[~nearS].mean())
clus=[]
for nn in range(1,17):
    for h in 'ab':
        X0,Y0=load('q2%02d%s'%(nn,h)); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
        clus+=[(X0[a:b],Y0[a:b]) for a,b in zip(br,br[1:]) if b-a>=256]
XC=np.concatenate([Xc for Xc,Yc in clus]).astype(LD); YC=np.concatenate([Yc for Xc,Yc in clus]); TC=sd(XC-LD(2)); UY=ulp(YC)
cid=np.concatenate([np.full(len(Xc),i) for i,(Xc,Yc) in enumerate(clus)]); cnt=np.bincount(cid); xs=np.array([float(Xc[len(Xc)//2]) for Xc,Yc in clus])
Dv=LD(Q0[0])+0*XC
for c in Q0[1:]: Dv=Dv*XC+LD(c)
SN=np.bincount(cid,np.array(TC*XC**4*LD(u)/Dv/UY,dtype=float))/cnt
PQ=np.load('SCRATCH/cand7_PQ.npy'); Pr=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
dr=np.array([(Pr[i]-P0[i])/u for i in (1,2,3,4,5)])
d=np.array((YC.astype(LD)-sd(TC*sd(hm(Pr,XC)/hm(Q,XC))))/UY,dtype=float); B=np.bincount(cid,d)/cnt
sB=np.sqrt(np.maximum(np.bincount(cid,d*d)/cnt-B*B,1e-8)/cnt)+1e-4; w=1/sB
print(f"reference: sample {sc(Pr,Q)}, bias rms {np.sqrt(np.mean(B**2)):.4f}; near-2 constraint value {16*dr[0]+8*dr[1]+4*dr[2]+2*dr[3]+dr[4]:.3f} u (target 0)")
g3=ulp(P0[3])/u; g2=ulp(P0[2])/u; g1=ulp(P0[1])/u
res=[]
for k in range(-24,25,2):
    d3=dr[2]+k*g3; c0=(7290-4*d3)/16
    base=SN*xs*((d3-dr[2])*xs + (c0-dr[0])*xs**3 - dr[1]*xs**2); slope=SN*xs*(xs**2 - 0.5*xs**3); tgt=B-base
    d2=np.sum((w*slope)*(w*tgt))/np.sum((w*slope)**2)
    for j in (-1,0,1):
        d2q=np.round(d2/g2)*g2 + j*g2; d1=(7290-4*d3-8*d2q)/16; d1q=np.round(d1/g1)*g1
        P=list(P0); P[1]=float(P0[1]+d1q*u); P[2]=float(P0[2]+d2q*u); P[3]=float(P0[3]+d3*u); P[4]=float(P0[4]-5209*u); P[5]=float(P0[5]+3128*u)
        s=sc(P,Q); fit=np.sqrt(np.mean((B-base-slope*d2q)**2)); res.append((s,k,d3,d2q,d1q,fit))
        print(f"  k={k:+3d} dp3={d3:+.3f} dp2={d2q:+.3f} dp1={d1q:+.4f} bias-fit rms {fit:.4f}  all {s[0]:.4f} near {s[1]:.4f} far {s[2]:.4f}",flush=True)
best=max(res); s,k,d3,d2q,d1q,fit=best; P=list(P0); P[1]=float(P0[1]+d1q*u); P[2]=float(P0[2]+d2q*u); P[3]=float(P0[3]+d3*u); P[4]=float(P0[4]-5209*u); P[5]=float(P0[5]+3128*u)
np.save('SCRATCH/cand9_PQ.npy',np.array([P,Q+[0.0]]))
for i in (1,2,3,4,5): print(f"   p{i}: {P[i].hex()}  delta {(P[i]-P0[i])/u:+.4f} u")
