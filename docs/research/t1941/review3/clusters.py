import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def chain(c,x,stop=None):
    s=LD(c[0])+0*x; st=[]
    for i in range(1,len(c)):
        P=sd(s*x); s=sd(P+LD(c[i])); st.append((P,s))
    return s,st
def full(p,q,x):
    n,_=chain(p,x); d,_=chain(q,x); t=sd(x-LD(2)); return sd(t*sd(n/d)).astype(np.float64)
def head(c,k,x):
    s=LD(c[0])+0*x
    for i in range(1,k+1): s=sd(sd(s*x)+LD(c[i]))
    return s
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def analyse(label,X,Y,p,q,CL=512):
    n=len(X); t=sd(X-LD(2)); s4=head(p,4,X); d3=head(q,3,X); u4=ulp(s4); u3=ulp(d3)
    adm={}
    for da in range(-2,3):
        nn=tail(p,4,sd(s4+da*u4),X)
        for db in range(-2,3):
            dd=tail(q,3,sd(d3+db*u3),X); adm[(da,db)]=(sd(t*sd(nn/dd)).astype(np.float64)==Y)
    need=~adm[(0,0)]
    # per-cluster best constant shift
    tot=0; best=0; signmix=0; ncl=0
    for a in range(0,n-CL+1,CL):
        sl=slice(a,a+CL); b=max(adm.values(),key=lambda m:m[sl].sum()); best+=b[sl].sum(); tot+=CL; ncl+=1
        plus=(need[sl]&adm[(1,0)][sl]).sum(); minus=(need[sl]&adm[(-1,0)][sl]).sum(); signmix+=min(plus,minus)
    # phase lock over all stages (sum phases), for points needing a shift
    _,stN=chain(p,X); _,stD=chain(q,X)
    out=[]
    for nm,st,c in (("N",stN,p),("D",stD,q)):
        for j,(P,s) in enumerate(st, start=1):
            if j<4: continue
            ph=np.array(((P+LD(c[j]))/ulp(s))%1,dtype=np.float64)
            h,_=np.histogram(ph[need],bins=4,range=(0,1)); hall,_=np.histogram(ph,bins=4,range=(0,1))
            r=h/np.maximum(hall,1)/need.mean(); r=r[hall>0]
            out.append(f"{nm}.S{j}:{r.max():.2f}/{r.min():.2f}")
    print(f"{label}: exact={adm[(0,0)].mean():.3f} need={need.mean():.3f} cluster-const-fit={best/tot:.3f} (over {ncl} clusters of {CL}); N+1/-1 sign-mixing per cluster={signmix/tot:.3f}")
    print("    lock (max/min relative rate over 4 phase quarters): "+" ".join(out))
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]
    analyse(case+" REAL",X,Y,P0,Q0)
    # synthetic truths scored against grid_a: (i) p3 +3 ulps, p2 -40 ulps (smooth head offset); (ii) p6 +12 ulps (late residue); (iii) q3 +1 ulp
    def pert(c,i,k): c=list(c); c[i]=c[i]+k*(np.nextafter(c[i],np.inf)-c[i]); return c
    for lab,pp,qq in (("SYN head p3+3,p2-40",pert(pert(P0,3,3),2,-40),Q0),("SYN late p6+12",pert(P0,6,12),Q0),("SYN q3+1,q2-6",pert(pert(Q0,3,1),2,-6),P0 and Q0)):
        if lab.startswith("SYN q3"): pp=P0; qq=pert(pert(Q0,3,1),2,-6)
        Ys=full(pp,qq,X); analyse(case+" "+lab,X,Ys,P0,Q0)
