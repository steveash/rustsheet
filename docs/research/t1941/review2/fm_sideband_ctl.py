# T-1941 review2: positive control for reading an INVISIBLE upstream product phase through the FM sidebands of a
# visible stage, by exact template matching on one dense run (K1, AN2 as truth).
# Parametrisation of a run's hidden state: a constant nudge d_k (fraction of u(P_k)) added to product k's
# pre-rounding value, i.e. the sub-ulp part of the upstream stage value S_{k-1}(y0)*y0 at this run.
# Truth = AN2 with (d2, d3) = (0.61, 0.37) on N products 2 and 3 (both invisible: output weight 0.01-0.08 ulp).
# Candidate scans (d2', d3') and scores exact matches against the truth's output on the run. If the exact count
# peaks sharply at (0.61, 0.37) the invisible phases are readable from one run; the control is then repeated with
# the candidate's D chain made wrong (D1 +1 ulp) to see whether the reading survives a wrong downstream.
import numpy as np, lib2, sys
from lib2 import L,st
N,D=lib2.load_cand_ND(lib2.AN2)
def horner_nudge(c,y,nud):
    """nud: dict stage->fraction of u(P_k) added to the product before its rounding (stage 1 = top product)."""
    s=np.full_like(y,c[-1]); k=0
    for ci in c[-2::-1]:
        k+=1
        pe=L(s)*L(y)
        if k in nud: pe=pe+L(nud[k])*np.spacing(np.abs(np.asarray(pe,dtype=np.float64)))
        p=st(pe); s=st(L(p)+L(ci))
    return s
def G(y,nN={},Dc=D,nD={}):
    return st(L(horner_nudge(N,y,nN))/L(horner_nudge(Dc,y,nD)))
y0=float(sys.argv[1]) if len(sys.argv)>1 else 1.8800
n=8192; y=y0+np.arange(n)*np.spacing(y0)
truth=G(y,{2:0.61,3:0.37})
grid=np.arange(0,1,1/32)
def scan(Dc,nD,label):
    best=(-1,None); M=np.zeros((len(grid),len(grid)))
    for i,d2 in enumerate(grid):
        for j,d3 in enumerate(grid):
            m=(G(y,{2:d2,3:d3},Dc,nD)==truth).mean(); M[i,j]=m
            if m>best[0]: best=(m,(d2,d3))
    i0=np.argmax(M.max(1)); j0=np.argmax(M.max(0))
    print(f"{label}: best exact {best[0]:.4f} at (d2,d3)={best[1]}; baseline (0,0) {M[0,0]:.4f}; "
          f"marginal over d3 at best d2: "+" ".join(f"{v:.3f}" for v in M[i0,::4])+f" | over d2 at best d3: "+" ".join(f"{v:.3f}" for v in M[::4,j0]))
    return M
scan(D,{},f"run y0={y0}, downstream exact")
D1=D[:]; D1[1]=D1[1]+np.spacing(abs(D1[1]))
scan(D1,{},f"run y0={y0}, candidate D1 +1 ulp (downstream wrong)")
scan(D,{5:0.5},f"run y0={y0}, candidate D product 5 phase off by 0.5")
