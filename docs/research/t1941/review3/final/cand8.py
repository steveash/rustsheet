# Joint integer coordinate search on a stratified sample: all near-2 points (x<2.02) + 400k random others. Structures S0 (stored) and S1 (M*).
import sys, numpy as np, time
sys.path.insert(0,'docs/research/t1941/review3')
from spec import p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(np.asarray(v,dtype=np.float64)); return (np.nextafter(a,np.inf)-a)
u=2.0**-54
X,Y=np.load('SCRATCH/allXY.npy')
rng=np.random.default_rng(1); near=X<2.02; far=np.where(~near)[0]; pick=rng.choice(far,400000,replace=False)
sel=np.concatenate([np.where(near)[0],pick]); X=X[sel]; Y=Y[sel]; near=X<2.02
XC=X.astype(LD); TC=sd(XC-LD(2))
def hm(c,x,unst):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        s=sd(s*x)+LD(c[i])
        if i not in unst: s=sd(s)
    return s
def sc(P,Q,unN,unD):
    e=sd(TC*sd(hm(P,XC,unN)/hm(Q,XC,unD))).astype(np.float64)==Y; return float(e.mean()), float(e[near].mean()), float(e[~near].mean())
PQ=np.load('SCRATCH/cand7_PQ.npy')   # starting set
for label,unN,unD in (('S1 (M*)',{3},{2}),('S0 (stored)',set(),set())):
    P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
    best=sc(P,Q,unN,unD); print(f"{label} start: all {best[0]:.4f} near {best[1]:.4f} far {best[2]:.4f}",flush=True)
    steps={'p4':(4,u),'p3':(3,ulp(P0[3])),'p2':(2,ulp(P0[2])),'p5':(5,u),'p1':(1,ulp(P0[1])*64),'q3':(3,u),'q2':(2,ulp(Q0[2])),'q1':(1,ulp(Q0[1])),'q4':(4,ulp(Q0[4]))}
    for rnd in range(3):
        moved=False
        for name,(idx,st) in steps.items():
            isP=name[0]=='p'; C=P if isP else Q; base=C[idx]; cand=[]
            for k in range(-12,13):
                if k==0: cand.append((best,0)); continue
                C2=list(C); C2[idx]=float(base+k*st); s=sc(C2,Q,unN,unD) if isP else sc(P,C2,unN,unD); cand.append((s,k))
            s,k=max(cand)
            if s[0]>best[0]: best=s; C[idx]=float(base+k*st); moved=True
            print(f"  {label} round {rnd} {name}: k={k:+d} -> all {best[0]:.4f} near {best[1]:.4f} far {best[2]:.4f}",flush=True)
        for (na,ia,nb,ib,ratio,rng_) in (('p',3,'p',2,-6.4,range(-8,9)),('q',2,'q',1,-12.8,range(-8,9))):   # null lines
            Ca=P if na=='p' else Q; Cb=P if nb=='p' else Q; sa=ulp(Ca[ia]); sb=ulp(Cb[ib]); res=[]
            for ka in rng_:
                for j in range(-3,4):
                    kb=int(round(ratio*ka))+j
                    if ka==0 and j==0: res.append((best,0,0)); continue
                    P2=list(P); Q2=list(Q); (P2 if na=='p' else Q2)[ia]=float(Ca[ia]+ka*sa); (P2 if nb=='p' else Q2)[ib]=float(Cb[ib]+kb*sb)
                    res.append((sc(P2,Q2,unN,unD),ka,kb))
            s,ka,kb=max(res)
            if s[0]>best[0]: best=s; Ca[ia]=float(Ca[ia]+ka*sa); Cb[ib]=float(Cb[ib]+kb*sb); moved=True
        if not moved: break
    print(f"{label} FINAL: all {best[0]:.4f} near {best[1]:.4f} far {best[2]:.4f}")
    for name,c0,c1 in (("p",P0,P),("q",Q0,Q)):
        for i,(a0,a1) in enumerate(zip(c0,c1)):
            if a0!=a1: print(f"   {name}{i}: {a1.hex()}  delta {(a1-a0)/u:+.2f} u")
    np.save('SCRATCH/cand8_%s.npy'%label[:2],np.array([P,Q+[0.0]]))
