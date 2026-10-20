# (HANDOFF 12.10) On the stored chain's s4 exact class: Excel's required s4 shift (A) and minimal admissible s3 shift (B),
# tabulated against P3_q+p3's eighth-position class, floor parity, sign(e3) and e3 in 1/16 bins. --control K plants p2 +K ulps.
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def ulpc(c): return np.nextafter(abs(c),np.inf)-abs(c)
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def simy(p,q,x): return sd(sd(x-LD(2))*sd(tail(p,0,LD(p[0])+0*x,x)/tail(q,0,LD(q[0])+0*x,x))).astype(np.float64)
args=sys.argv[1:]; ctl=None
if args and args[0]=='--control': ctl=int(args[1]); args=args[2:]
for case in args:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=len(X)
    if ctl is not None:
        pt=list(P0); pt[2]=pt[2]+ctl*ulpc(pt[2]); Y=simy(pt,list(Q0),X)
    s=LD(P0[0])+0*X; vals=[]
    for i in range(1,5): P80=s*X; Pq=sd(P80); v=Pq+LD(P0[i]); s=sd(v); vals.append((P80,Pq,v,s))
    P3_80,P3q,v3,s3=vals[2]; P4_80,P4q,v4,s4=vals[3]
    u4=ulp(s4); pos4=np.array(v4/u4-np.floor(v4/u4),dtype=np.float64); exact=np.isclose(pos4,0.0)
    u3=ulp(s3); q3=v3/u3; fl3=np.floor(q3); pos3=np.array(q3-fl3,dtype=np.float64); par3=(np.array(fl3,dtype=np.float64).astype(np.int64)&1)
    uP3=ulp(P3q); e3=np.array((P3_80-P3q)/uP3,dtype=np.float64)
    D=tail(Q0,0,LD(Q0[0])+0*X,X)
    yd=sd(t*sd(tail(P0,4,sd(s4-u4),X)/D)).astype(np.float64); yu=sd(t*sd(tail(P0,4,sd(s4+u4),X)/D)).astype(np.float64); y0=sd(t*sd(tail(P0,4,s4,X)/D)).astype(np.float64)
    md=Y==yd; mu=Y==yu; m0=Y==y0; A=np.where(m0,0,np.where(mu&~md,1,np.where(md&~mu,-1,9)))
    B=np.full(n,9)
    for k in (0,1,-1,2,-2,3,-3):
        yk=sd(t*sd(tail(P0,3,sd(s3+k*u3),X)/D)).astype(np.float64); hit=(yk==Y)&(B==9); B[hit]=k
    cls=np.round(pos3*8).astype(int)%8
    print(f"== {case} x0={float(X[n//2]):.4f}{' CONTROL p2%+d' % ctl if ctl is not None else ''}: exact-class points {exact.sum()}, s4: 0 {np.mean(A[exact]==0):.3f} +1 {np.mean(A[exact]==1):.3f} -1 {np.mean(A[exact]==-1):.3f} undecided {np.mean(A[exact]==9):.3f}; s3 minimal shift: 0 {np.mean(B[exact]==0):.3f} +-1 {np.mean(np.abs(B[exact])==1):.3f} +-2 {np.mean(np.abs(B[exact])==2):.3f} +-3 {np.mean(np.abs(B[exact])==3):.3f} none {np.mean(B[exact]==9):.3f}")
    for c in range(8):
        row=[]
        for par in (0,1):
            for sg,sel_e in (("e3<0",e3<0),("e3>0",e3>0)):
                sel=exact&(cls==c)&(par3==par)&sel_e; m=sel.sum()
                row.append(f"{'ev' if par==0 else 'od'} {sg}: --" if m<15 else f"{'ev' if par==0 else 'od'} {sg}: {np.mean(A[sel]==1):.2f}/{np.mean(A[sel]==-1):.2f}/{np.mean((B[sel]!=0)&(B[sel]!=9)):.2f}({m})")
        print(f"   pos {c}/8: "+"  ".join(row))
    bins=np.clip(((e3+0.5)*16).astype(int),0,15)
    print("   s3-shift rate by e3 bin: "+" ".join(f"{np.mean((B[exact&(bins==bb)]!=0)&(B[exact&(bins==bb)]!=9)):.2f}" if (exact&(bins==bb)).sum()>=15 else "--" for bb in range(16)))
    sel=exact&(np.abs(B)==1)
    if sel.sum()>=15: print(f"   among s3 +-1 shifts (n={sel.sum()}): sign(k)==sign(e3) {np.mean(np.sign(B[sel])==np.sign(e3[sel])):.3f}")
