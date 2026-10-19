# T-1941 review2: is the exact-match landscape along ONE K1 coefficient a sawtooth with period u(S_k) (the sum's ulp),
# i.e. a lattice of near-equivalent points? Scan c_k over +-2.5 u(S_k) in steps of u(S_k)/16 and score exact match on
# three y windows. If the lattice picture is right the near-2 window shows peaks at every multiple of u(S_k) with a
# strong envelope, and the near-1 window is flat.
import numpy as np, lib2, sys
from lib2 import L,st
N,D=lib2.load_cand_ND(lib2.AN2)
x,g,_=lib2.g_rows()
rng=np.random.default_rng(1)
def win(lo,hi,n=60000):
    s=np.where((x>=lo)&(x<hi))[0]; s=rng.choice(s,min(n,len(s)),replace=False); return x[s],g[s]
wins={'1.00-1.10':win(1.0,1.1),'1.50-1.60':win(1.5,1.6),'1.90-2.00':win(1.9,2.0)}
which=sys.argv[1] if len(sys.argv)>1 else 'N'
stages=[int(v) for v in sys.argv[2].split(',')] if len(sys.argv)>2 else [1,3,5]
for k in stages:
    c=(N if which=='N' else D)[:]; idx=len(c)-1-k; c0=c[idx]
    # u(S_k) at y=1.5 from a trace
    tr=[]; lib2.horner_asc(c,np.array([1.5]),trace=tr); uS=np.spacing(abs(tr[k-1][1][0])); uc=np.spacing(abs(c0))
    print(f"\n{which} stage {k}: constant c_{len(c)-1-k} ulp(c)=2^{int(np.log2(uc))}, u(S_k)@1.5=2^{int(np.log2(uS))}, period = {uS/uc:.0f} ulps of c")
    steps=np.arange(-40,41)/16.0
    print("  delta/u(S)  "+"  ".join(f"{w:>9s}" for w in wins))
    for t in steps:
        c[idx]=c0+t*uS
        row=[]
        for w,(yy,gg) in wins.items():
            sim=lib2.G_AN2(N if which=='D' else c, D if which=='N' else c, yy) if False else (lib2.G_AN2(c,D,yy) if which=='N' else lib2.G_AN2(N,c,yy))
            row.append((sim==gg).mean())
        print(f"  {t:+6.3f}      "+"  ".join(f"{r:9.4f}" for r in row))
