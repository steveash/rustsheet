# Step 1: per-run profiles R_r(delta mod 2) of the exact-match rate under s4 = RNE(v4 + delta*u) - round(delta)*u
# (equivalently RN(v4 + f u) with ties to even/odd), delta on the 1/8 grid in [0,2); same for D's d3.
# Step 2: global search for integer-ulp head offsets (D4,D3,D2,D1) with delta4(x) = D4 + D3 x/8 + D2 x^2/128 + D1 x^3/16384
# maximising the summed profile over runs; compare with the per-run optimum (upper bound) and the stored baseline.
import sys, numpy as np, time
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
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
DELTA=np.arange(16)/8.0   # delta mod 2
prof={}; xs={}
for case in RUNS:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=len(X)
    s=LD(P0[0])+0*X
    for i in range(1,4): s=sd(sd(s*X)+LD(P0[i]))
    v4=sd(s*X)+LD(P0[4]); s4=sd(v4); u=ulp(s4)
    d=LD(Q0[0])+0*X
    for i in range(1,3): d=sd(sd(d*X)+LD(Q0[i]))
    v3=sd(d*X)+LD(Q0[3]); d3=sd(v3); uD=ulp(d3)
    def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
    def variant(v,g,delta):   # delta in [0,2): f = delta mod 1 mapped to [-0.5,0.5), parity = floor(delta) after mapping
        k=np.floor(delta+0.5); f=delta-k; return rn(v+f*g,g,bool(int(k)%2))
    pN=np.array([np.mean(yof(variant(v4,u,dl),d3)==Y) for dl in DELTA])
    pD=np.array([np.mean(yof(s4,variant(v3,uD,dl))==Y) for dl in DELTA])
    prof[case]=(pN,pD); xs[case]=float(X[n//2])
    print(f"{case} x={xs[case]:.4f} stored {pN[0]:.3f} | N profile "+" ".join(f"{v:.3f}" for v in pN)+f" | D profile "+" ".join(f"{v:.3f}" for v in pD), flush=True)
np.save('profiles.npy',{'prof':prof,'xs':xs},allow_pickle=True)   # (scratch path in my copy)
x=np.array([xs[c] for c in RUNS]); PN=np.array([prof[c][0] for c in RUNS]); PD=np.array([prof[c][1] for c in RUNS])
base=PN[:,0].sum(); ubN=PN.max(1).sum(); ubD=PD.max(1).sum()
print(f"baseline sum {base:.3f}; per-run optimum sum N {ubN:.3f} (+{ubN-base:.3f}), D {ubD:.3f} (+{ubD-base:.3f})")
def gsearch(P,basis,ranges,label,top=8):
    B=np.array([b(x) for b in basis]); nr=len(x)
    grids=[np.arange(lo,hi+1,st) for lo,hi,st in ranges]
    bestlist=[]
    G=np.meshgrid(*grids[:-1],indexing='ij'); flat=np.stack([g.ravel() for g in G],1); last=grids[-1]
    t0=time.time()
    for row in flat:
        baseD=row@B[:-1]                                   # nr
        pred=baseD[None,:]+last[:,None]*B[-1][None,:]      # nlast x nr
        idx=np.rint(np.mod(pred,2.0)*8).astype(int)%16
        sc=P[np.arange(nr)[None,:],idx].sum(1)
        k=np.argmax(sc); bestlist.append((float(sc[k]),tuple(int(v) for v in row)+(int(last[k]),)))
    bestlist.sort(reverse=True)
    print(f"{label} [{time.time()-t0:.0f}s]: top solutions (summed exact rate over {nr} runs; baseline {P[:,0].sum():.3f}, per-run optimum {P.max(1).sum():.3f}):")
    for sc,co in bestlist[:top]: print(f"   sum {sc:.3f} (+{sc-P[:,0].sum():.3f})  coeffs {co}")
    return bestlist[0]
bN=gsearch(PN,[lambda x:np.ones_like(x),lambda x:x/8,lambda x:x**2/128],[(-3,3,1),(-64,64,1),(-2000,2000,1)],"N (D4,D3,D2)")
bN4=gsearch(PN,[lambda x:np.ones_like(x),lambda x:x/8,lambda x:x**2/128,lambda x:x**3/16384],[(-3,3,1),(-40,40,1),(-600,600,4),(-6000,6000,1)],"N (D4,D3,D2,D1)")
bD=gsearch(PD,[lambda x:0.5*np.ones_like(x),lambda x:x/16,lambda x:x**2/512],[(-3,3,1),(-64,64,1),(-2000,2000,1)],"D (D3,D2,D1)")
bD4=gsearch(PD,[lambda x:0.5*np.ones_like(x),lambda x:x/16,lambda x:x**2/512,lambda x:x**3/65536],[(-3,3,1),(-40,40,1),(-600,600,4),(-20000,20000,4)],"D (D3,D2,D1,D0)")
for label,co,basis,P in (("N",bN4[1],[lambda x:1+0*x,lambda x:x/8,lambda x:x**2/128,lambda x:x**3/16384],PN),("D",bD4[1],[lambda x:0.5+0*x,lambda x:x/16,lambda x:x**2/512,lambda x:x**3/65536],PD)):
    pred=sum(c*b(x) for c,b in zip(co,basis)); idx=np.rint(np.mod(pred,2.0)*8).astype(int)%16
    print(f"{label} best {co}: per run (x, delta mod 2, rate, per-run opt, stored): "+"; ".join(f"{RUNS[i]}({x[i]:.3f}) {np.mod(pred[i],2):.2f} {P[i,idx[i]]:.3f}/{P[i].max():.3f}/{P[i,0]:.3f}" for i in range(len(x))))
