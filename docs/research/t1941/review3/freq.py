import os, sys, math, numpy as np
from fractions import Fraction as F
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, sim, exact, ulp, p, q, step
LD=np.longdouble
def slopes(c,x,k):  # returns S_k, dS_k (exact), ulp
    S=F(c[0]); dS=F(0)
    for i in range(1,k+1):
        P=S*x; dP=dS*x+S; S=P+F(c[i]); dS=dP
    u=F(2)**(math.floor(math.log2(abs(float(S))))-52); return S,dS,u
def refine(r,f0,w=0.004,steps=801):
    # fine scan of |DFT| around f0 (sub-bin), returns best f and power; r is mean-removed
    n=len(r); k=np.arange(n); fs=np.linspace(f0-w,f0+w,steps)
    best=(0,f0,0)
    for f in fs:
        z=np.dot(r,np.exp(-2j*np.pi*f*k)); pw=abs(z)**2/n
        if pw>best[0]: best=(pw,f,np.angle(z))
    return best
MINLEN=int(sys.argv[1])
for case in sys.argv[2:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    for a,b in zip(br,br[1:]):
        if b-a<MINLEN: continue
        X=X0[a:b]; Y=Y0[a:b]; n=len(X); ys=sim(X); ex=exact(X); u=ulp(Y)
        rE=((Y.astype(LD)-ex)/u.astype(LD)).astype(np.float64); rS=((ys.astype(LD)-ex)/u.astype(LD)).astype(np.float64)
        rE-=rE.mean(); rS-=rS.mean(); med=np.median(np.abs(np.fft.rfft(rE))**2/n)
        x0=F(X[n//2])
        for nm,c,k,cj in (("N.S4",p,4,p[1]),("D.S3",q,3,q[0])):
            S,dS,uu=slopes(c,x0,k); fb=float((dS*step/uu)%1); alias=fb>0.5; fbA=1-fb if alias else fb
            dshift=float(3*F(cj)*x0*x0*step/uu)  # slope of the x^3-from-top term c_{k-3} x^3 : for N.S4 that is p1*x^3, for D.S3 q0*x^3
            fMC = fbA - dshift if not alias else fbA + dshift
            pe,fe,phe=refine(rE,fbA); ps,fs_,phs=refine(rS,fbA)
            # second harmonic
            pe2,fe2,_=refine(rE,min(2*fbA,1-2*fbA),w=0.006); ps2,fs2,_=refine(rS,min(2*fbA,1-2*fbA),w=0.006)
            print(f"{case} x0={float(x0):.5f} n={n} {nm}: f_base={fbA:.5f} f_MC={fMC:.5f} | Excel f={fe:.5f} ({pe/med:6.0f}x) sim f={fs_:.5f} ({ps/med:6.0f}x) dphase={((phe-phs+np.pi)%(2*np.pi)-np.pi)/(2*np.pi):+.3f}cyc | 2nd harm: Excel {fe2:.5f} ({pe2/med:5.0f}x) sim {fs2:.5f} ({ps2/med:5.0f}x) [2f_base={min(2*fbA,1-2*fbA):.5f}]")
