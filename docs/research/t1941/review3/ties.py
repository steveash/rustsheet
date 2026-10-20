# Direct tie-class test at the s4 sum (and d3). On points where the stored-chain sum P4_q + p4 is an exact tie at the
# u grid, Excel's output decides whether Excel rounded down or up (when the two differ downstream). Compare Excel's
# choice with: ties-to-even (the x87 rule and the sim), the true side of the unquantised product P4_80, always-up,
# always-down. Then the same for the quarter classes (position 1/4 and 3/4), where RN is unambiguous.
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def upto(c,k,x):
    s=LD(c[0])+0*x
    for i in range(1,k): s=sd(sd(s*x)+LD(c[i]))
    return s*x
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; n=len(X); t=sd(X-LD(2))
    for nm,c,k,other in (("N s4",P0,4,None),("D d3",Q0,3,None)):
        P80=upto(c,k,X); Pq=sd(P80); v=Pq+LD(c[k]); s=sd(v); u=ulp(s)
        q=v/u; fl=np.floor(q); pos=np.array(q-fl,dtype=np.float64)   # exact in 64-bit: v/u < 2^53 and on a 1/8 grid
        sdown=fl*u; sup=sdown+u   # the two neighbours
        sdown=sd(sdown); sup=sd(sup)
        if nm.startswith("N"):
            D=tail(Q0,0,LD(Q0[0])+0*X,X)  # full D
            yd=sd(t*sd(tail(c,k,sdown,X)/D)).astype(np.float64); yu=sd(t*sd(tail(c,k,sup,X)/D)).astype(np.float64)
        else:
            N=tail(P0,0,LD(P0[0])+0*X,X)
            yd=sd(t*sd(N/tail(c,k,sdown,X))).astype(np.float64); yu=sd(t*sd(N/tail(c,k,sup,X))).astype(np.float64)
        md=(Y==yd); mu=(Y==yu); decided=md^mu
        side=np.array(((P80+LD(c[k]))/u - fl),dtype=np.float64)>0.5   # true side of the unquantised sum (above midpoint?)
        even_up=(np.array(fl%2,dtype=np.float64))!=0                # ties-to-even chooses up when the lower neighbour is odd
        excel_up=mu&decided
        print(f"== {case} x0={float(X[n//2]):.4f} {nm}: tie class {np.mean(np.isclose(pos,0.5)):.3f} of points; quarter classes {np.mean(np.isclose(pos,0.25)):.3f}/{np.mean(np.isclose(pos,0.75)):.3f}; other {np.mean(~(np.isclose(pos,0.5)|np.isclose(pos,0.25)|np.isclose(pos,0.75)|np.isclose(pos,0.0))):.3f}")
        for cls,sel in (("tie 1/2",np.isclose(pos,0.5)),("pos 1/4",np.isclose(pos,0.25)),("pos 3/4",np.isclose(pos,0.75)),("pos 0",np.isclose(pos,0.0))):
            d=sel&decided
            if d.sum()==0: continue
            agree_even=np.mean(excel_up[d]==even_up[d]); agree_side=np.mean(excel_up[d]==side[d]); up=np.mean(excel_up[d])
            rn_up = (pos[d]>0.5) if cls!="tie 1/2" else even_up[d]
            print(f"   {cls}: decided {d.sum():5d} of {sel.sum():5d}; Excel up {up:.3f}; agrees with RN/even {np.mean(excel_up[d]==rn_up):.3f}, with true side of P80 {agree_side:.3f}; neither-neighbour {np.mean(~(md|mu)[sel]):.3f}")
