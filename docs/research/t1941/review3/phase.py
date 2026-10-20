# Model-free, continuous read of the hand-off offset: the PHASE of the N.S4 (and D.S3) sawtooth line in Excel-exact
# relative to sim-exact gives delta4 mod 1 u directly (a pre-rounding offset delta shifts the sawtooth by delta/u cycles).
# Calibrated per run with a planted +0.25 u offset. Done for the long runs and for W2's 1,024 dense clusters; then the
# neighbour test on the continuous estimate (smooth delta => adjacent clusters agree).
import sys, numpy as np, pickle
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0, fS, exact
from fractions import Fraction as F
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def amp(r,f):
    k=np.arange(len(r)); return np.dot(r-r.mean(),np.exp(-2j*np.pi*f*k))
def fS_signed(c,x,k):   # signed fractional frequency (cycles/point), not folded
    import math
    S=F(c[0]); dS=F(0)
    for i in range(1,k+1):
        P=S*x; dP=dS*x+S; S=P+F(c[i]); dS=dP
    u=F(2)**(math.floor(math.log2(abs(float(S))))-52); f=float((dS*F(2)**-51/u)%1); return f-1 if f>0.5 else f
def analyse(X0,Y0,a,b):
    X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=b-a
    s=LD(P0[0])+0*X
    for i in range(1,4): s=sd(sd(s*X)+LD(P0[i]))
    v4=sd(s*X)+LD(P0[4]); s4=sd(v4); u=ulp(s4)
    d=LD(Q0[0])+0*X
    for i in range(1,3): d=sd(sd(d*X)+LD(Q0[i]))
    v3=sd(d*X)+LD(Q0[3]); d3=sd(v3); uD=ulp(d3)
    def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
    ex=exact(X0[a:b]); uy=ulp(Y.astype(LD))
    def res(y): return np.array((y.astype(LD)-ex)/uy,dtype=np.float64)
    rE=res(Y); rS=res(yof(s4,d3)); rCN=res(yof(sd(v4+0.25*u),d3)); rCD=res(yof(s4,sd(v3+0.25*uD)))
    x0=F(X0[a+n//2]); f4=fS_signed(P0,x0,4); f3=fS_signed(Q0,x0,3)
    out={}
    for lab,f,rC in (("N",f4,rCN),("D",f3,rCD)):
        AE,AS,AC=amp(rE,f),amp(rS,f),amp(rC,f)
        dphiC=np.angle(AC/AS)/(2*np.pi); sgn=np.sign(dphiC) if abs(dphiC)>0.02 else 1.0
        dphiE=np.angle(AE/AS)/(2*np.pi)
        out[lab]=(sgn*dphiE, abs(AE)/abs(AS), dphiC, abs(AS)**2/n)   # offset in u (mod 1, in (-0.5,0.5]), amplitude ratio, calibration shift, line power
    return out
print("LONG RUNS: run x | N: phase offset (u, mod 1) amp ratio [calib +0.25 -> measured] | D: same")
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
for case in RUNS:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); o=analyse(X0,Y0,a,b)
    print(f"  {case:6s} x={float(X0[(a+b)//2]):.4f} | N: {o['N'][0]:+.3f} u, amp {o['N'][1]:.2f} [calib {o['N'][2]:+.3f}] | D: {o['D'][0]:+.3f} u, amp {o['D'][1]:.2f} [calib {o['D'][2]:+.3f}]", flush=True)
print("DENSE CLUSTERS")
rows=pickle.load(open('clusters.pkl','rb'))   # written by clusterfit.py
cache={}; recs=[]
for r in rows:
    k,case,a,n,x=r[0],r[1],r[2],r[3],r[4]
    if case not in cache: cache[case]=load(case)
    X0,Y0=cache[case]; o=analyse(X0,Y0,a,a+n)
    recs.append((k,x,o['N'][0],o['N'][1],o['D'][0],o['D'][1],r[6],r[7]-r[5],r[8]))
recs.sort(); pickle.dump(recs,open('phases.pkl','wb'))
x=np.array([r[1] for r in recs]); pN=np.array([r[2] for r in recs]); aN=np.array([r[3] for r in recs]); pD=np.array([r[4] for r in recs]); aD=np.array([r[5] for r in recs])
def circ(a,b): d=np.mod(a-b,1.0); return np.minimum(d,1-d)
rng=np.random.default_rng(0)
for lab,ph in (("N",pN),("D",pD)):
    adj=circ(ph[1:],ph[:-1]); rnd=circ(ph[rng.permutation(len(ph))],ph)
    print(f"  {lab}: phase offsets mod 1 u: mean |adjacent diff| {adj.mean():.3f} (random pairs {rnd.mean():.3f}; uniform would be 0.25); fraction |adj diff|<0.1: {np.mean(adj<0.1):.2f} (random {np.mean(rnd<0.1):.2f})")
    print(f"     histogram of offsets (bins of 0.1 from -0.5): {np.histogram(ph,bins=np.arange(-0.5,0.51,0.1))[0]}; mean amp ratio {np.mean(aN if lab=='N' else aD):.2f}")
fN=np.array([r[6] for r in recs]); gN=np.array([r[7] for r in recs]); sel=gN>=0.03
print(f"  N: among clusters with rule gain>=0.03 (n={sel.sum()}): mean circular |phase - f| = {circ(pN[sel],np.mod(fN[sel],1.0)).mean():.3f} (random {circ(pN[sel],np.mod(fN[sel][rng.permutation(sel.sum())],1.0)).mean():.3f})")
# law fit used afterwards: restrict to 0.7 < amp ratio < 1.4 (and x < 2.63 for N); maximise |mean exp(2πi(ph − B·g(x)))| over B for g = 1/x etc.
