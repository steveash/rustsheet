# Class x parity table of Excel's required s4 shift (D fixed) and the "2u grid at the hand-off" variants.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0, fS
from fractions import Fraction as F
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def rn_grid(v,g):      # round v (LD, exact) to the grid g (LD, power of two), ties to even on that grid
    q=v/g; f=np.floor(q); r=q-f; up=(r>0.5)|((r==0.5)&(np.mod(f,2)==1)); return (f+up)*g
for case in sys.argv[1:]:
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
    y0=yof(s4,d3); yu=yof(sd(s4+u),d3); yd=yof(sd(s4-u),d3)
    A=np.where(y0==Y,0,np.where((yu==Y)&(yd!=Y),1,np.where((yd==Y)&(yu!=Y),-1,9)))
    q=v4/u; fl=np.floor(q); pos=np.array(q-fl,dtype=np.float64); par=(np.array(fl,dtype=np.float64).astype(np.int64)&1)
    print(f"== {case} x0={float(X[n//2]):.4f} n={n} exact {np.mean(y0==Y):.3f}; D fixed; cells: pos x floor parity -> frac +1 / -1 / undecided (n)")
    for pz in (0.0,0.25,0.5,0.75):
        row=[]
        for pr in (0,1):
            sel=np.isclose(pos,pz)&(par==pr); m=sel.sum()
            row.append(f"par {pr}: {np.mean(A[sel]==1):.2f}/{np.mean(A[sel]==-1):.2f}/{np.mean(A[sel]==9):.2f} ({m})" if m else f"par {pr}: -- (0)")
        print(f"   pos {pz:.2f}: "+"   ".join(row))
    # variants
    s4g=rn_grid(v4,2*u); d3g=rn_grid(v3,2*uD)
    print(f"   variants exact: s4 on 2u grid {np.mean(yof(s4g,d3)==Y):.3f}; d3 on 2u grid {np.mean(yof(s4,d3g)==Y):.3f}; both {np.mean(yof(s4g,d3g)==Y):.3f}; (stored {np.mean(y0==Y):.3f})")
    yv=yof(s4g,d3); diff=yv!=y0
    print(f"   s4-2u variant differs from stored on {diff.mean():.3f}; Excel follows variant {np.mean(yv[diff]==Y[diff]):.3f}, stored {np.mean(y0[diff]==Y[diff]):.3f}, neither {np.mean((yv[diff]!=Y[diff])&(y0[diff]!=Y[diff])):.3f}")
    # half-frequency line check
    from spec import exact
    exv=exact(X0[a:b]); uy=ulp(Y.astype(LD))
    rE=np.array((Y.astype(LD)-exv)/uy,dtype=np.float64); rS=np.array((y0.astype(LD)-exv)/uy,dtype=np.float64)
    freqs=np.fft.rfftfreq(n); PE=np.abs(np.fft.rfft(rE-rE.mean()))**2/n; PS=np.abs(np.fft.rfft(rS-rS.mean()))**2/n
    f4=fS(P0,F(X0[a+n//2]),4); med=np.median(PE)
    for nm,f in (("N.S4",f4),("N.S4/2",f4/2),("N.S4*3/2",min(1.5*f4,1-1.5*f4) if 1.5*f4<1 else 1.5*f4-1)):
        m=(freqs>f-0.0015)&(freqs<f+0.0015)
        if m.any():
            i=np.argmax(PE[m]); j=np.argmax(PS[m])
            print(f"   line {nm:8s} f={f:.5f}: Excel-exact {PE[m][i]/med:7.1f}x at {freqs[m][i]:.5f} | sim-exact {PS[m][j]/np.median(PS):7.1f}x at {freqs[m][j]:.5f}")
