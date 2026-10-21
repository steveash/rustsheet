# Residual spectra of Excel GAMMALN.PRECISE on dense K2 runs against (a) the exact grid_a rational and (b) the exact x87
# stored-Horner simulation of grid_a. Reports the power at the base N.S4 / D.S3 sum-line frequencies and at the
# candidate x-independent pair-sum lines 8*p3 and 8*q2, relative to the median spectral power.
import json, os, sys, math
import numpy as np
from fractions import Fraction as F
LD = np.longdouble
p=[float.fromhex(h) for h in "0x1.5717994ef45d1p-51 0x1.5e5b73810edd1p-16 0x1.0ee2f118ad254p-9 0x1.572bf8683b589p-5 0x1.09fd1ad1da445p-2 0x1.f0a8c05397a9fp-2 -0x1.08305b47998b9p-6 -0x1.0ae34eb3cbad6p-1 -0x1.de852aee8c153p-3 -0x1.22260ed97843dp-6".split()]
q=[float.fromhex(h) for h in "0x1.03cadf2db0d9dp-18 0x1.52fc9914ab713p-11 0x1.5303658ef66c8p-6 0x1.bb43b464d3259p-3 0x1.c515177eb1755p-1 0x1.7b39fc9194438p+0 0x1.eddf2040d2bc2p-1 0x1.9b252b2fab347p-3 0x1.00000000000eap-7".split()]
def load(case):
    d='eval/suites/probes-numeric/cases/kmyst-gam-%s'%case
    c=json.load(open(d+'.case.json'))['sheets'][0]['cells']; v=json.load(open(d+'.oracle.json'))['sheets']['Calc']
    X=[];Y=[]; r=1
    while 'A%d'%r in c:
        y=v.get('B%d'%r,{}).get('v')
        if isinstance(y,(int,float)): X.append(float(c['A%d'%r])); Y.append(float(y))
        r+=1
    return np.array(X), np.array(Y)
def sd(v): return v.astype(np.float64).astype(LD)
def sim(X):  # stored x87 Horner, y = fl(t*fl(N/D))
    x=X.astype(LD); n=LD(p[0])+0*x; d=LD(q[0])+0*x
    for c in p[1:]: n=sd(sd(n*x)+LD(c))
    for c in q[1:]: d=sd(sd(d*x)+LD(c))
    return sd(sd(x-LD(2))*sd(n/d)).astype(np.float64)
def exact(X):  # exact rational in float64-valued ulps; use 80-bit Horner of exact coefficients (error << 1e-3 ulp)
    x=X.astype(LD); n=LD(p[0])+0*x; d=LD(q[0])+0*x
    for c in p[1:]: n=n*x+LD(c)
    for c in q[1:]: d=d*x+LD(c)
    return (x-LD(2))*n/d
def ulp(y): return np.nextafter(np.abs(y),np.inf)-np.abs(y)
def power_at(r,f):
    n=len(r); k=np.arange(n); z=np.exp(-2j*np.pi*f*k); return abs(np.dot(r-r.mean(),z))**2/n
def peak_near(P,freqs,f0,w):
    m=(freqs>f0-w)&(freqs<f0+w); i=np.argmax(P[m]); return freqs[m][i], P[m][i]
step=F(2)**-51
def fS(c,x,k):  # base sum-line freq of stage k (1-based) at x, aliased
    S=F(c[0]); dS=F(0)
    for i in range(1,k+1):
        P=S*x; dP=dS*x+S; S=P+F(c[i]); dS=dP
    u=F(2)**(math.floor(math.log2(abs(float(S))))-52); f=float((dS*step/u)%1); return min(f,1-f)
if __name__=="__main__":
    for case in sys.argv[1:]:
        X,Y=load(case)
        # keep the longest consecutive stretch
        br=[0]+[i+1 for i in range(len(X)-1) if np.nextafter(X[i],np.inf)!=X[i+1]]+[len(X)]
        a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X[a:b]; Y=Y[a:b]
        ys=sim(X); ex=exact(X); u=ulp(Y)
        rE=((Y.astype(LD)-ex)/u.astype(LD)).astype(np.float64)   # Excel - exact
        rS=((ys.astype(LD)-ex)/u.astype(LD)).astype(np.float64)  # base sim - exact
        rD=((Y-ys)/u)                                               # Excel - sim
        n=len(X); freqs=np.fft.rfftfreq(n); 
        PE=np.abs(np.fft.rfft(rE-rE.mean()))**2/n; PS=np.abs(np.fft.rfft(rS-rS.mean()))**2/n; PD=np.abs(np.fft.rfft(rD-rD.mean()))**2/n
        medE=np.median(PE); medS=np.median(PS)
        x0=F(X[n//2]); fN4=fS(p,x0,4); fD3=fS(q,x0,3); fN3=fS(p,x0,3); fN5=fS(p,x0,5)
        fA_N=float((F(p[3])*step/F(2)**-54)%1); fA_D=float((F(q[2])*step/F(2)**-54)%1)
        print(f"{case} x0={float(x0):.4f} n={n} exact={np.mean(Y==ys):.3f} rms(E-ex)={rE.std():.3f} rms(S-ex)={rS.std():.3f} medP E/S={medE:.2f}/{medS:.2f}")
        for nm,f in (("N.S4",fN4),("D.S3",fD3),("N.S3",fN3),("N.S5",fN5),("pairN 8p3",fA_N),("pairD 8q2",fA_D)):
            fe,pe=peak_near(PE,freqs,f,0.0015); fs_,ps=peak_near(PS,freqs,f,0.0015); fd,pd=peak_near(PD,freqs,f,0.0015)
            print(f"   {nm:10s} f={f:.5f}: Excel-exact peak {pe/medE:7.1f}x at {fe:.5f} | sim-exact {ps/medS:7.1f}x at {fs_:.5f} | Excel-sim {pd/medE:7.1f}x at {fd:.5f}")
        # top Excel-only peaks
        top=np.argsort(PE)[::-1][:12]
        print("   top Excel-exact lines:", " ".join(f"{freqs[i]:.5f}({PE[i]/medE:.0f}x)" for i in sorted(top, key=lambda i:-PE[i])))
        top=np.argsort(PS)[::-1][:12]
        print("   top sim-exact lines:  ", " ".join(f"{freqs[i]:.5f}({PS[i]/medS:.0f}x)" for i in sorted(top, key=lambda i:-PS[i])))
