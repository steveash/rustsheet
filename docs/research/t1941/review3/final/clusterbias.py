# Model-free bias curve: per-cluster mean(Excel − sim)/ulp(y) over the 1,024 clusters -> s5-level offset e5(x) = B·ulp(y)·D/(t·x^4) in u;
# weighted LS fits with the constant pinned at +3128u: quadratic Δ (x,x²,x³) vs cubic Δ (x..x⁴) vs controls. Saves cubic_coef.npy (a1..a4 of e5−3128).
import sys, numpy as np, pickle
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0, exact
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(np.asarray(v,dtype=np.float64)); return (np.nextafter(a,np.inf)-a)
u=2.0**-54
def horner(c,x):
    s=LD(c[0])+0*x
    for i in range(1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
rows=[]
for nn in range(1,17):
    for h in 'ab':
        X0,Y0=load('q2%02d%s'%(nn,h)); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
        for a,b in zip(br,br[1:]):
            if b-a<256: continue
            X=X0[a:b]; Y=Y0[a:b]; Xl=X.astype(LD); t=sd(Xl-LD(2)); n=b-a; uy=ulp(Y)
            sim=sd(t*sd(horner(P0,Xl)/horner(Q0,Xl)))   # replace P0,Q0 by the set under test
            dR=np.array((Y.astype(LD)-sim)/uy,dtype=float)
            Dv=LD(Q0[0])+0*Xl
            for c in Q0[1:]: Dv=Dv*Xl+LD(c)
            SN=float(np.mean(np.array(t*Xl**4*LD(u)/Dv/uy,dtype=float)))
            rows.append((float(X[n//2]),dR.mean(),dR.std()/np.sqrt(n),SN))
R=np.array(rows); x=R[:,0]; mR=R[:,1]; sR=R[:,2]; SN=R[:,3]; e5=mR/SN; w=1/np.maximum(sR/SN,1e-9)
def fit(cols,target):
    A=np.vstack(cols).T*w[:,None]; coef,*_=np.linalg.lstsq(A,target*w,rcond=None); res=target-np.vstack(cols).T@coef; return coef,res
for name,cols,tgt in (("quadratic Delta (pinned)",[x,x**2,x**3],e5-3128),("cubic Delta (pinned)",[x,x**2,x**3,x**4],e5-3128),("control free cubic",[x**0,x,x**2,x**3],e5)):
    coef,res=fit(cols,tgt); print(f"{name:26s} rms {np.sqrt(np.mean(res**2)):.3f} u chi2/dof {np.sum((res*w)**2)/(len(x)-len(cols)):.2f} coef {np.array2string(coef,precision=2)}")
    if name.startswith("cubic"): np.save('SCRATCH/cubic_coef.npy',coef)
