# T-1941 review2: the re-hash cascade, quantified. For each kernel, a +1 ulp change of ONE coefficient c_j; for every
# stored intermediate downstream, the fraction of points at which the stored double differs from the unperturbed
# chain. Rows: perturbed coefficient (top of the chain first). Columns: stored values S_1..S_n (stage sums).
# A fraction near 0.5 at a stage means that stage's rounding sequence is fully re-hashed by the move; that is the
# "jitter" of HANDOFF section 7 and the reason single-phase meters of a stage fail when its upstream is wrong.
import numpy as np, lib2, os
from lib2 import L,st
def chain_trace(c_desc,x,bump=None):
    """c_desc highest degree first; every op stored; returns list of stored sums. bump=(j,ulps) on c_desc[j]."""
    c=list(c_desc)
    if bump: c[bump[0]]=c[bump[0]]+bump[1]*np.spacing(abs(c[bump[0]]))
    s=np.full_like(x,c[0]); out=[]
    for ci in c[1:]:
        p=st(L(s)*L(x)); s=st(L(p)+L(ci)); out.append(s)
    return out
def report(name,c_desc,x):
    base=chain_trace(c_desc,x); n=len(base)
    print(f"\n{name}: fraction of points where stored S_k differs after +1 ulp on coefficient (highest degree = index 0); n={len(x)}")
    print("  coef   "+" ".join(f"S{k+1:<4d}" for k in range(n)))
    for j in range(len(c_desc)):
        pert=chain_trace(c_desc,x,(j,1))
        print(f"  c[{j}]  "+" ".join(f"{(pert[k]!=base[k]).mean():5.3f}" for k in range(n)))
rng=np.random.default_rng(5)
N,D=lib2.load_cand_ND(lib2.AN2)
x,_,_=lib2.g_rows(); s=rng.choice(np.where(x>1.8)[0],20000,replace=False); y=x[s]
report("K1 AN2 N chain, y in (1.8,2)",N[::-1],y); report("K1 AN2 D chain, y in (1.8,2)",D[::-1],y)
s=rng.choice(np.where(x<1.2)[0],20000,replace=False); y=x[s]
report("K1 AN2 N chain, y in [1,1.2)",N[::-1],y)
X,B,C,cs=lib2.ln_rows()
P=Q=None
for l in open(os.path.join(lib2.HERE,'..','w2','efC.txt')):
    p=l.split()
    if p and p[0]=='p': P=[float.fromhex(v) for v in p[1:]]
    if p and p[0]=='q': Q=[float.fromhex(v) for v in p[1:]]
s=rng.choice(np.where((X>2.4)&(X<3.0))[0],20000,replace=False); xx=X[s]
report("K2 efC N chain, x in (2.4,3)",P,xx); report("K2 efC D chain, x in (2.4,3)",Q,xx)
ls=[l.split() for l in open(os.path.join(lib2.HERE,'..','w5','p4','ss_fc.txt')) if l.strip() and not l.startswith('#')]
N3=[float.fromhex(v) for v in ls[0]]; D3=[float.fromhex(v) for v in ls[1]]
s=rng.choice(np.where((X>0.7)&(X<1.47))[0],20000,replace=False); xx=X[s]
report("K3 ss_fc N chain, x in (0.7,1.47)",N3[::-1],xx); report("K3 ss_fc D chain",D3[::-1],xx)
