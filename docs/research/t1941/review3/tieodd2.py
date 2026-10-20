# After the best hand-off rule (bias + tie parity) on s4 and d3: (1) residual class table, (2) the same rule family
# applied one stage earlier (s3, d2) on top, (3) the same on the first late sums (s5, d4).
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def rn(v,g,odd):
    q=v/g; f=np.floor(q); r=q-f; fo=np.mod(f,2)==1
    up=(r>0.5)|((r==0.5)&(fo if not odd else ~fo)); return (f+up)*g
F=np.arange(-8,9)/8.0
def chain(c,x,rules,upto=None):
    # rules: {stage: (f, odd)} applied at that stage's sum rounding; other stages RNE via sd
    s=LD(c[0])+0*x; K=len(c)-1 if upto is None else upto
    for i in range(1,K+1):
        v=sd(s*x)+LD(c[i])
        if i in rules:
            f,o=rules[i]; g=ulp(sd(v)); s=rn(v+f*g,g,o)
        else: s=sd(v)
    return s
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=len(X)
    def yof(rN,rD): return sd(t*sd(chain(P0,X,rN)/chain(Q0,X,rD))).astype(np.float64)
    y0=yof({},{}); base=np.mean(y0==Y)
    # best hand-off rule (search N then D then N)
    rN={}; rD={}
    for _ in range(2):
        cand={(f,o):np.mean(yof({4:(f,o)},rD)==Y) for f in F for o in (False,True)}; kN=max(cand,key=cand.get); rN={4:kN}
        cand={(g,o):np.mean(yof(rN,{3:(g,o)})==Y) for g in F for o in (False,True)}; kD=max(cand,key=cand.get); rD={3:kD}
    y1=yof(rN,rD); r1=np.mean(y1==Y)
    print(f"== {case} x={float(X[n//2]):.4f} n={n}: stored {base:.3f} -> hand-off rules N{rN[4]} D{rD[3]}: {r1:.3f}")
    # residual class table on the NEW chain's s4 geometry
    s3=chain(P0,X,{},3); v4=sd(s3*X)+LD(P0[4]); u=ulp(sd(v4)); s4n=chain(P0,X,rN,4)
    q=v4/u; fl=np.floor(q); pos=np.array(q-fl,dtype=np.float64); par=(np.array(fl,dtype=np.float64).astype(np.int64)&1)
    d3n=chain(Q0,X,rD,3)
    def yfrom(s4v,d3v):
        s=s4v
        for i in range(5,10): s=sd(sd(s*X)+LD(P0[i]))
        d=d3v
        for i in range(4,9): d=sd(sd(d*X)+LD(Q0[i]))
        return sd(t*sd(s/d)).astype(np.float64)
    yu=yfrom(sd(s4n+u),d3n); yd=yfrom(sd(s4n-u),d3n)
    rows=[]
    for pz in (0.0,0.25,0.5,0.75):
        for pr in (0,1):
            sel=np.isclose(pos,pz)&(par==pr)
            if sel.sum()==0: continue
            vu=sel&(yu!=y1); vd=sel&(yd!=y1)
            rows.append(f"({pz:.2f},{pr}): up {np.mean(Y[vu]==yu[vu]):.2f} dn {np.mean(Y[vd]==yd[vd]):.2f} miss {np.mean(y1[sel]!=Y[sel]):.2f}")
    print("   residual by class [P(Excel=+1|vis), P(Excel=-1|vis), miss]: "+"; ".join(rows))
    # one stage earlier
    cand={(f,o):np.mean(yof({**rN,3:(f,o)},rD)==Y) for f in F for o in (False,True)}; k3=max(cand,key=cand.get)
    cand2={(g,o):np.mean(yof(rN,{**rD,2:(g,o)})==Y) for g in F for o in (False,True)}; k2=max(cand2,key=cand2.get)
    r3=np.mean(yof({**rN,3:k3},{**rD,2:k2})==Y)
    print(f"   + s3 rule {k3}: {cand[k3]:.3f};  + d2 rule {k2}: {cand2[k2]:.3f};  both: {r3:.3f}")
    # first late sums
    cand={(f,o):np.mean(yof({**rN,5:(f,o)},rD)==Y) for f in F for o in (False,True)}; k5=max(cand,key=cand.get)
    cand2={(g,o):np.mean(yof(rN,{**rD,4:(g,o)})==Y) for g in F for o in (False,True)}; k4=max(cand2,key=cand2.get)
    print(f"   + s5 rule {k5}: {cand[k5]:.3f};  + d4 rule {k4}: {cand2[k4]:.3f}")
