# Two-polynomial model (NEGATIVE result, kept for the record): Excel's N = fl( Horner_stored(hi, x) + Horner_stored(lo, x) ),
# hi = grid_a + Delta on the head (p1..p4), lo = -Delta, late hi = grid_a; hand-off constant's integer part m scanned. 0.25-0.60.
import sys, numpy as np, math
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulpf(c): return 2.0**(math.floor(math.log2(abs(c)))-52)
def horner(c,x):
    s=LD(c[0])+0*x
    for i in range(1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def horner80(c,x):
    s=LD(c[0])+0*x
    for i in range(1,len(c)): s=s*x+LD(c[i])
    return sd(s)
u=2.0**-54
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
data={}
for case in RUNS:
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); data[case]=(X0[a:b].astype(LD),Y0[a:b])
def build(c,head,K,target_off,m):
    off_head=sum(head[j]*ulpf(c[j])*2.25**(K-j) for j in head)/u; dK=(target_off+m-off_head)
    hi=list(c); lo=[0.0]*len(c)
    for j in head: hi[j]=float(np.float64(c[j]+head[j]*ulpf(c[j]))); lo[j]=float(np.float64(c[j]-hi[j]))
    hi[K]=float(np.float64(c[K]+dK*u)); lo[K]=float(np.float64(c[K]-hi[K]))
    return hi,lo
def yof(hiN,loN,hiD,loD,X,mode):
    t=sd(X-LD(2)); N=horner(hiN,X); D=horner(hiD,X)
    if mode=='two': N=sd(N+horner80(loN,X)); D=sd(D+horner80(loD,X))
    return sd(t*sd(N/D)).astype(np.float64)
def score(hiN,loN,hiD,loD,mode='two'):
    r={c:np.mean(yof(hiN,loN,hiD,loD,X,mode)==Y) for c,(X,Y) in data.items()}; return r,np.mean(list(r.values()))
r0,m0=score(P0,[0]*10,Q0,[0]*9,'one'); print("grid_a one-Horner: "+" ".join(f"{c}:{v:.3f}" for c,v in r0.items())+f" | mean {m0:.3f}")
headN={1:1693013,2:-124864,3:27519}; headD={0:-873813,1:69888,2:-8340}
best=None
for mN in range(-2,3):
    hiN,loN=build(P0,headN,4,-0.116,mN); r,mean=score(hiN,loN,Q0,[0]*9); print(f"N two-poly mN={mN:+d}: mean {mean:.3f}")
    if best is None or mean>best[0]: best=(mean,mN,hiN,loN)
bestD=None
for mD in range(-2,3):
    hiD,loD=build(Q0,headD,3,0.463,mD); r,mean=score(best[2],best[3],hiD,loD); print(f"D two-poly mD={mD:+d}: mean {mean:.3f}")
    if bestD is None or mean>bestD[0]: bestD=(mean,mD,hiD,loD)
print(f"best joint mean {bestD[0]:.3f} at mN={best[1]}, mD={bestD[1]}")
