import json, os, sys, math
import numpy as np
from fractions import Fraction as F
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, sim, exact, ulp, fS, p, q, step
MINLEN=int(sys.argv[1]); cases=sys.argv[2:]
fA_N=float((F(p[3])*step/F(2)**-54)%1); fA_D=float((F(q[2])*step/F(2)**-54)%1)
for case in cases:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    for a,b in zip(br,br[1:]):
        if b-a<MINLEN: continue
        X=X0[a:b]; Y=Y0[a:b]; n=len(X)
        ys=sim(X); ex=exact(X); u=ulp(Y); LD=np.longdouble
        rE=((Y.astype(LD)-ex)/u.astype(LD)).astype(np.float64); rS=((ys.astype(LD)-ex)/u.astype(LD)).astype(np.float64)
        freqs=np.fft.rfftfreq(n); PE=np.abs(np.fft.rfft(rE-rE.mean()))**2/n; PS=np.abs(np.fft.rfft(rS-rS.mean()))**2/n
        mE=np.median(PE); mS=np.median(PS)
        x0=F(X[n//2]); f4=fS(p,x0,4); f3=fS(q,x0,3)
        def pk(P,m,f,w=0.0008):
            msk=(freqs>f-w)&(freqs<f+w); i=np.argmax(P[msk]); return P[msk][i]/m, freqs[msk][i]
        e4,fe4=pk(PE,mE,f4); s4,fs4=pk(PS,mS,f4); e3,fe3=pk(PE,mE,f3); s3,fs3=pk(PS,mS,f3)
        eA,feA=pk(PE,mE,fA_N); sA,fsA=pk(PS,mS,fA_N); eD,feD=pk(PE,mE,fA_D); sD,fsD=pk(PS,mS,fA_D)
        print(f"{case} x0={float(x0):.5f} n={n} exact={np.mean(Y==ys):.3f} | N.S4 f={f4:.5f}: E {e4:6.1f}x S {s4:6.1f}x | D.S3 f={f3:.5f}: E {e3:6.1f}x S {s3:6.1f}x | 8p3 {fA_N:.5f}: E {eA:6.1f}x({feA:.5f}) S {sA:5.1f}x | 8q2 {fA_D:.5f}: E {eD:6.1f}x({feD:.5f}) S {sD:5.1f}x")
        # Excel-only and sim-only strong lines
        rat=(PE/mE)/(PS/mS+1); top=np.argsort(PE/mE)[::-1][:40]
        eo=[i for i in top if PE[i]/mE>15 and PS[i]/mS<4][:8]; so=[i for i in np.argsort(PS/mS)[::-1][:40] if PS[i]/mS>15 and PE[i]/mE<4][:8]
        print("     Excel-only:", " ".join(f"{freqs[i]:.5f}({PE[i]/mE:.0f}x)" for i in eo), " | sim-only:", " ".join(f"{freqs[i]:.5f}({PS[i]/mS:.0f}x)" for i in so))
