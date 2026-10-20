# R5 residual selector tables: within hot (pos,parity) cells at s4, P(flip | visible) by the sign/magnitude of the product residue e_P and by the s3 position.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
u=2.0**-54; CN=-3128*u; CD=552*u
def chainR5(c,x,K,cc,upto=None):
    s=LD(c[0])+0*x; n=len(c)-1 if upto is None else upto
    for i in range(1,n+1):
        if i==K: s=sd(sd(s*x+LD(cc)/x)+LD(c[i]))
        elif i==K+1: s=sd(sd(s*x)+LD(c[i]-cc))
        else: s=sd(sd(s*x)+LD(c[i]))
    return s
def tail(c,k,s,x,cc):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i])-(LD(cc) if i==k+1 else 0))
    return s
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
for case in RUNS:
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2))
    D=chainR5(Q0,X,3,CD); s3=chainR5(P0,X,4,CN,upto=3); Pex=s3*X+LD(CN)/X; P4=sd(Pex); eP=np.array((Pex-P4)/ulp(P4),dtype=np.float64)
    v4=P4+LD(P0[4]); s4=sd(v4); g=ulp(s4)
    def yfrom(s4v): return sd(t*sd(tail(P0,4,s4v,X,CN)/D)).astype(np.float64)
    y1=yfrom(s4); yu=yfrom(sd(s4+g)); yd=yfrom(sd(s4-g))
    q=v4/g; fl=np.floor(q); pos=np.array(q-fl,dtype=np.float64); par=(np.array(fl,dtype=np.float64).astype(np.int64)&1)
    v3=sd(chainR5(P0,X,4,CN,upto=2)*X)+LD(P0[3]); q3=v3/ulp(sd(v3)); pos3=np.array(q3-np.floor(q3),dtype=np.float64); out=[]
    for pz in np.unique(np.round(pos*8)/8):
        for pr in (0,1):
            sel=(np.abs(pos-pz)<1e-6)&(par==pr)
            if sel.sum()<40: continue
            vu=sel&(yu!=y1); vd=sel&(yd!=y1); up=np.mean(Y[vu]==yu[vu]) if vu.any() else 0; dn=np.mean(Y[vd]==yd[vd]) if vd.any() else 0
            if max(up,dn)<0.15: continue
            flip=(vu&(Y==yu)) if up>dn else (vd&(Y==yd)); vis=vu if up>dn else vd
            r_pos=np.mean(flip[vis&(eP>0)]) if (vis&(eP>0)).any() else float('nan'); r_neg=np.mean(flip[vis&(eP<0)]) if (vis&(eP<0)).any() else float('nan')
            out.append(f"({pz:.2f},{pr}) {'up' if up>dn else 'dn'} {max(up,dn):.2f}: eP>0 {r_pos:.2f} eP<0 {r_neg:.2f} | pos3<.5/>=.5 {np.mean(flip[vis&(pos3<0.5)]):.2f}/{np.mean(flip[vis&(pos3>=0.5)]):.2f}")
    print(f"== {case} R5 {np.mean(y1==Y):.3f}: "+" ; ".join(out), flush=True)
