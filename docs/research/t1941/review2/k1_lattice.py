# T-1941 review2: K1 (AN2) stage table. For each Horner stage k (1 = top): binade relation of P_k vs S_k, the lattice
# period u(S_k)/u(c_k) (how many ulps of the constant c_k move the stored sum by exactly one of its ulps, i.e. the
# flip-free period), and the output weight of a one-ulp change of S_k, in ulps of G, at several y.
import numpy as np, lib2
from lib2 import L,st
N,D=lib2.load_cand_ND(lib2.AN2)
ys=np.array([1.02,1.2,1.4,1.6,1.75,1.85,1.95,1.99])
def table(c,name):
    tr=[]; s=lib2.horner_asc(c,ys,trace=tr)
    print(f"\n{name} chain, constants (ascending index) ulp(c_k):")
    for k,(p,sv) in enumerate(tr,1):
        ck=c[-1-k]   # constant added at stage k
        print(f"  stage {k:d} c={ck:+.6e} ulp(c)=2^{int(np.log2(np.spacing(abs(ck)))):d}  "
              +"  ".join(f"y={y:.2f}: |P|/|S|={abs(pp)/abs(ss):5.2f} u(S)/u(c)=2^{int(np.log2(np.spacing(abs(ss))/np.spacing(abs(ck)))):d}" for y,pp,ss in zip(ys,p,sv)))
    return tr,s
trN,n=table(N,'N'); trD,d=table(D,'D')
g=st(L(n)/L(d)); ug=np.spacing(g)
print("\nOutput weight of +1 ulp on stored S_k (ulps of G), by y:")
print("        "+"  ".join(f"y={y:.2f}" for y in ys))
for k in range(1,8):
    gn=lib2.G_AN2(N,D,ys,nN=(k,'s',1)); print(f"  N.S{k}  "+"  ".join(f"{v:+6.2f}" for v in (gn-g)/ug))
for k in range(1,8):
    gd=lib2.G_AN2(N,D,ys,nD=(k,'s',1)); print(f"  D.S{k}  "+"  ".join(f"{v:+6.2f}" for v in (gd-g)/ug))
print("Output weight of +1 ulp on stored P_k:")
for k in range(1,8):
    gn=lib2.G_AN2(N,D,ys,nN=(k,'p',1)); print(f"  N.P{k}  "+"  ".join(f"{v:+6.2f}" for v in (gn-g)/ug))
for k in range(1,8):
    gd=lib2.G_AN2(N,D,ys,nD=(k,'p',1)); print(f"  D.P{k}  "+"  ".join(f"{v:+6.2f}" for v in (gd-g)/ug))
