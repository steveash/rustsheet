# Model-free read of Excel's effective hand-off rounding function for N (s4) and D (d3): P(Excel = floor+1) vs the
# position of the exact (80-bit unstored) head value on the u grid, split by floor parity; reports the 0.5-crossing
# (threshold) per parity per run, with grid_a's own chain as the RNE reference (its crossing shows the smear).
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def crossing(pos,up,bins=16):
    # P(up) per bin, then linear interpolation of the 0.5 crossing (after rescaling between the low and high plateaus)
    P=np.array([np.mean(up[(pos>=k/bins)&(pos<(k+1)/bins)]) for k in range(bins)]); c=(np.arange(bins)+0.5)/bins
    lo=np.nanmean(P[:2]); hi=np.nanmean(P[-2:]); Q=(P-lo)/(hi-lo) if hi>lo else P
    idx=np.where((Q[:-1]<0.5)&(Q[1:]>=0.5))[0]
    if len(idx)==0: return float('nan'),P
    i=idx[-1]; return float(c[i]+(0.5-Q[i])/(Q[i+1]-Q[i])*(c[i+1]-c[i])),P
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
print("run x | N: thr_odd thr_even (ref_odd ref_even) | D: thr_odd thr_even (ref_odd ref_even) | unknown N/D")
for case in RUNS:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=len(X)
    s=LD(P0[0])+0*X
    for i in range(1,4): s=sd(sd(s*X)+LD(P0[i]))
    s4=sd(sd(s*X)+LD(P0[4])); u=ulp(s4)
    h=LD(P0[0])+0*X
    for i in range(1,5): h=h*X+LD(P0[i])
    d=LD(Q0[0])+0*X
    for i in range(1,3): d=sd(sd(d*X)+LD(Q0[i]))
    d3=sd(sd(d*X)+LD(Q0[3])); uD=ulp(d3)
    hD=LD(Q0[0])+0*X
    for i in range(1,4): hD=hD*X+LD(Q0[i])
    def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
    y0=yof(s4,d3); out=[f"{case:6s} {float(X[n//2]):.4f}"]
    unk=[]
    for lab,sv,uu,hh,dv,which in (("N",s4,u,h,d3,0),("D",d3,uD,hD,s4,1)):
        yu=yof(sd(sv+uu),dv) if which==0 else yof(dv,sd(sv+uu)); yd=yof(sd(sv-uu),dv) if which==0 else yof(dv,sd(sv-uu))
        qh=hh/uu; fl=np.floor(qh); pos=np.array(qh-fl,dtype=np.float64); par=(np.array(fl,dtype=np.float64).astype(np.int64)&1)
        off=np.array((sv-fl*uu)/uu,dtype=np.float64)
        exc=np.where(y0==Y,off,np.where(yu==Y,off+1,np.where(yd==Y,off-1,np.nan)))
        ok=~np.isnan(exc); unk.append(1-ok.mean())
        thr=[]; ref=[]
        for pr in (1,0):
            sel=ok&(par==pr); c,_=crossing(pos[sel],(exc[sel]==1).astype(float)); thr.append(c)
            selr=(par==pr); cr,_=crossing(pos[selr],(off[selr]==1).astype(float)); ref.append(cr)
        out.append(f"{lab}: {thr[0]:.3f} {thr[1]:.3f} ({ref[0]:.3f} {ref[1]:.3f})")
    print(" | ".join(out)+f" | {unk[0]:.3f}/{unk[1]:.3f}", flush=True)
