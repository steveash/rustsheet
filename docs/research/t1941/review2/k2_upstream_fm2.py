# T-1941 review2: K2 single-phase meter under upstream vs downstream errors, properly controlled.
# Truths (all from efC, function change printed): B = upstream p7 -2, p6 +3, q7 +2, q6 -3 ulps;
# C = downstream p2 +1, q1 -1 ulps; A = efC itself (meter sanity: readings 0, r = 1).
# Meter: at an anchor run, scan a sum nudge on the metered stage (N s4, N s5, D s3, D s4); reading = nudge that
# maximises the correlation between residual (truth - cand)/ulp and template (cand nudged - cand)/ulp. The metered
# stages' own constants are unchanged in every truth, so the correct reading is 0.
import numpy as np, lib2, os
from lib2 import L,st
X,B,C,cs=lib2.ln_rows()
P=Q=None
for l in open(os.path.join(lib2.HERE,'..','w2','efC.txt')):
    p=l.split()
    if p and p[0]=='p': P=np.array([float.fromhex(v) for v in p[1:]])
    if p and p[0]=='q': Q=np.array([float.fromhex(v) for v in p[1:]])
def chain(cd,x,sn=None):
    s=np.full_like(x,cd[0])
    for k,ci in enumerate(cd[1:]):
        pe=L(s)*L(x); p=st(pe); se=L(p)+L(ci)
        if sn is not None and sn.get(k+1,0): se=se+L(sn[k+1])*np.spacing(np.abs(np.asarray(se,dtype=np.float64)))
        s=st(se)
    return s
def k2(x,P,Q,snN=None,snQ=None):
    S=st(L(chain(P,x,snN))/L(chain(Q,x,snQ))); m=st(L(x)-L(2.0)); return st(L(m)*L(S))
def runs_at(a):
    out=[]
    for nm in sorted(set(cs)):
        s=np.where(cs==nm)[0]; o=np.argsort(X[s]); xs=X[s][o]
        d=np.diff(xs); br=np.where(d!=np.spacing(xs[:-1]))[0]
        for seg in np.split(np.arange(len(xs)),br+1):
            if len(seg)>=4096 and abs(xs[seg[0]]-a)<2e-3: out.append((nm,xs[seg][:8192]))
    return out[:1]
up=lambda arr,moves: arr+np.array(moves)*np.spacing(np.abs(arr))
truths={'A efC':(P,Q),'B upstream p7-2 p6+3 q7+2 q6-3':(up(P,[0,-2,3,0,0,0,0,0,0,0]),up(Q,[0,2,-3,0,0,0,0,0,0])),
        'C downstream p2+1 q1-1':(up(P,[0,0,0,0,0,0,0,1,0,0]),up(Q,[0,0,0,0,0,0,0,-1,0]))}
grid=np.arange(0,1,1/32)
for tl,(Pt,Qt) in truths.items():
    print("\nTRUTH:",tl)
    for a in (2.125,2.25,2.5,2.75,2.875):
        for nm,y in runs_at(a):
            tgt=k2(y,Pt,Qt); g0=k2(y,P,Q); u=np.spacing(np.abs(g0)); e=(tgt-g0)/u; ex0=(g0==tgt).mean()
            out=[]
            for stage,which in ((5,'N'),(4,'N'),(4,'D'),(3,'D')):
                best=(-9,0)
                for ph in grid:
                    gg=k2(y,P,Q,{stage:ph} if which=='N' else None,{stage:ph} if which=='D' else None); T=(gg-g0)/u
                    if T.std()==0 or e.std()==0: continue
                    sc=np.corrcoef(e,T)[0,1]
                    if sc>best[0]: best=(sc,ph)
                out.append(f"{which}s{stage}: {((best[1]+0.5)%1-0.5):+.3f} (r={best[0]:.2f})")
            print(f"  anchor {a} {nm[10:]} exact {ex0:.3f} max|res| {np.abs(e).max():.2f} mean res {e.mean():+.3f}: "+"  ".join(out),flush=True)
