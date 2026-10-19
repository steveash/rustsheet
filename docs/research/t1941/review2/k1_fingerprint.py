# T-1941 review2: near y=2, does a single hidden-stage flip leave a stage-specific signature in the output error?
# For each stage, nudge its stored value by +1 ulp on a dense sample in [1.85,2) and tabulate the distribution of the
# output change in ulps of G (mean, std, and the share of points where it is 0, i.e. invisible). Also the histogram of
# Excel - AN2 there.
import numpy as np, lib2
from lib2 import L,st
N,D=lib2.load_cand_ND(lib2.AN2)
x,g,_=lib2.g_rows(); rng=np.random.default_rng(2)
for lo,hi in ((1.85,1.9),(1.9,1.95),(1.95,2.0),(1.0,1.05)):
    s=np.where((x>=lo)&(x<hi))[0]; s=rng.choice(s,min(40000,len(s)),replace=False); yy=x[s]; gg=g[s]
    g0=lib2.G_AN2(N,D,yy); ug=np.spacing(g0); e=np.rint((gg-g0)/ug).astype(int)
    h={v:(e==v).mean() for v in range(-4,5)}
    print(f"\ny in [{lo},{hi}): Excel-AN2 histogram (ulps): "+" ".join(f"{v:+d}:{p:.3f}" for v,p in h.items())+f"  exact {h[0]:.3f}")
    print("  stage  mean   std   P(0)  P(|d|>=2)   (output change for +1 ulp on the stored value)")
    for ch,nm in ((('N',),'N'),(('D',),'D')):
        for kind in 'ps':
            for k in range(1,8):
                gs=lib2.G_AN2(N,D,yy,nN=(k,kind,1)) if nm=='N' else lib2.G_AN2(N,D,yy,nD=(k,kind,1))
                dd=(gs-g0)/ug
                print(f"  {nm}.{kind.upper()}{k}  {dd.mean():+6.2f} {dd.std():5.2f}  {(dd==0).mean():.3f}  {(abs(dd)>=2).mean():.3f}")
