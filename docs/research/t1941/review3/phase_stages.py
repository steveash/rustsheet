# Where does the boundary displacement live? Measure the sawtooth-line phase shift (Excel vs sim, calibrated with a
# planted +0.25 ulp at that stage's sum) for N stages 3,4,5,6 and D stages 2,3,4 on the long runs and the clusters.
import sys, numpy as np, pickle, math
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0, exact
from fractions import Fraction as F
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def amp(r,f):
    k=np.arange(len(r)); return np.dot(r-r.mean(),np.exp(-2j*np.pi*f*k))
def fS_signed(c,x,k):
    S=F(c[0]); dS=F(0)
    for i in range(1,k+1):
        P=S*x; dP=dS*x+S; S=P+F(c[i]); dS=dP
    u=F(2)**(math.floor(math.log2(abs(float(S))))-52); f=float((dS*F(2)**-51/u)%1); return f-1 if f>0.5 else f
def chain_off(c,x,stage,off):   # stored Horner with a pre-rounding offset off*ulp at the given stage's sum
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        v=sd(s*x)+LD(c[i])
        if i==stage: v=v+off*ulp(sd(v))
        s=sd(v)
    return s
def analyse(X0,Y0,a,b,stagesN=(3,4,5,6),stagesD=(2,3,4)):
    X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=b-a
    ex=exact(X0[a:b]); uy=ulp(Y.astype(LD))
    def res(y): return np.array((y.astype(LD)-ex)/uy,dtype=np.float64)
    def yof(N,D): return sd(t*sd(N/D)).astype(np.float64)
    N0=chain_off(P0,X,0,0); D0=chain_off(Q0,X,0,0)
    rE=res(Y); rS=res(yof(N0,D0)); x0=F(X0[a+n//2]); out={}
    for st in stagesN:
        f=fS_signed(P0,x0,st); rC=res(yof(chain_off(P0,X,st,0.25),D0))
        AE,AS,AC=amp(rE,f),amp(rS,f),amp(rC,f); dC=np.angle(AC/AS)/(2*np.pi); sgn=np.sign(dC) if abs(dC)>0.02 else 0.0
        out[('N',st)]=(sgn*np.angle(AE/AS)/(2*np.pi), abs(AE)/abs(AS), dC, abs(AS)**2/n/np.median(np.abs(np.fft.rfft(rS-rS.mean()))**2/n))
    for st in stagesD:
        f=fS_signed(Q0,x0,st); rC=res(yof(N0,chain_off(Q0,X,st,0.25)))
        AE,AS,AC=amp(rE,f),amp(rS,f),amp(rC,f); dC=np.angle(AC/AS)/(2*np.pi); sgn=np.sign(dC) if abs(dC)>0.02 else 0.0
        out[('D',st)]=(sgn*np.angle(AE/AS)/(2*np.pi), abs(AE)/abs(AS), dC, abs(AS)**2/n/np.median(np.abs(np.fft.rfft(rS-rS.mean()))**2/n))
    return out
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704"]
print("LONG RUNS: per stage: phase shift (ulp of that stage's sum, mod 1) [calib] (line power/median)")
for case in RUNS:
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); o=analyse(X0,Y0,a,b)
    print(f"  {case:6s} x={float(X0[(a+b)//2]):.4f} | "+" | ".join(f"{k[0]}{k[1]}: {v[0]:+.3f} [{v[2]:+.2f}] ({v[3]:.0f}x)" for k,v in o.items()), flush=True)
print("CLUSTERS: per-cluster phase shift per stage (use calibration |dC|>0.12 and 0.5<amp ratio<2 as the reliability filter)")
rows=pickle.load(open('clusters.pkl','rb'))
cache={}; acc={}
for r in rows:
    k,case,a,n=r[0],r[1],r[2],r[3]
    if case not in cache: cache[case]=load(case)
    X0,Y0=cache[case]; o=analyse(X0,Y0,a,a+n,stagesN=(3,4,5),stagesD=(2,3,4))
    for kk,v in o.items(): acc.setdefault(kk,[]).append((r[4],)+v)
pickle.dump(acc,open('phase_stages.pkl','wb'))
def circ(a,b): d=np.mod(a-b,1.0); return np.minimum(d,1-d)
for kk,lst in acc.items():
    A=np.array(lst); ok=(np.abs(A[:,3])>0.12)&(A[:,2]>0.5)&(A[:,2]<2.0); ph=A[ok,1]
    if len(ph)<20: print(f"  {kk}: only {len(ph)} reliable clusters"); continue
    z=np.exp(2j*np.pi*ph); R=abs(z.mean()); mean=np.angle(z.mean())/(2*np.pi)
    xs=A[ok,0]; order=np.argsort(xs); ph=ph[order]; adj=circ(ph[1:],ph[:-1]); rnd=circ(ph[np.random.default_rng(0).permutation(len(ph))],ph)
    print(f"  {kk[0]}{kk[1]}: {ok.sum()} reliable clusters; mean shift {mean:+.3f} ulp, resultant {R:.2f}; |adjacent diff| {adj.mean():.3f} vs random {rnd.mean():.3f}")
