# T-1941 review2: single-coefficient +-6 ulp scans of the MID coefficients of K3's ss_fc and K2's grid_a, scored by
# exact match on a 60k sample. Motivation: the re-hash matrix shows that one ulp of a mid coefficient re-hashes
# 20-50% of the final sum, so the exact-match basin there is a few ulps wide (W5 p5), and if the truth lies within
# that box a plain scan finds it. A flat scan says the remaining error is elsewhere (upstream, or sub-grid bits).
import numpy as np, lib2, os, sys
from lib2 import L,st
X,B,C,cs=lib2.ln_rows(); rng=np.random.default_rng(9)
def horner(cd,x):
    s=np.full_like(x,cd[0])
    for ci in cd[1:]: s=st(L(st(L(s)*L(x)))+L(ci))
    return s
ls=[l.split() for l in open(os.path.join(lib2.HERE,'..','w5','p4','ss_fc.txt')) if l.strip() and not l.startswith('#')]
N3=np.array([float.fromhex(v) for v in ls[0]])[::-1]; D3=np.array([float.fromhex(v) for v in ls[1]])[::-1]   # highest first
def k3(x,N,D): A=st(L(horner(N,x))/L(horner(D,x))); t=st(L(x)-L(1.0)); return st(L(t)*L(A))
P=Q=None
for l in open(os.path.join(lib2.HERE,'..','w2','grid_a.txt')):
    p=l.split()
    if p and p[0]=='p': P=np.array([float.fromhex(v) for v in p[1:]])
    if p and p[0]=='q': Q=np.array([float.fromhex(v) for v in p[1:]])
def k2(x,P,Q): S=st(L(horner(P,x))/L(horner(Q,x))); m=st(L(x)-L(2.0)); return st(L(m)*L(S))
for name,lo,hi,f,A,Bc,idxA,idxB in (("K3 ss_fc",0.7,1.47,k3,N3,D3,range(2,8),range(2,8)),("K2 grid_a",2.05,3.0,k2,P,Q,range(1,7),range(1,6))):
    s=np.where((X>=lo)&(X<hi)&(np.abs(X-1)>2e-3))[0]; s=rng.choice(s,60000,replace=False); x=X[s]; b=B[s]
    base=(f(x,A,Bc)==b).mean(); print(f"\n{name}: base exact {base:.4f} on {len(x)} pts in [{lo},{hi})")
    for lab,arr,idx in (("N/p",A,idxA),("D/q",Bc,idxB)):
        for j in idx:
            row=[]
            for d in range(-6,7):
                a2=arr.copy(); a2[j]=arr[j]+d*np.spacing(abs(arr[j]))
                sc=(f(x,a2,Bc)==b).mean() if lab[0]=='N' else (f(x,A,a2)==b).mean()
                row.append(sc)
            print(f"  {lab}[{j}] (deg {len(arr)-1-j}): "+" ".join(f"{v:.3f}" for v in row)+("   <-- max off-centre" if np.argmax(row)!=6 else ""),flush=True)
