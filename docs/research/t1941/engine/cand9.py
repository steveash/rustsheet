import sys, numpy as np
S=sys.argv[1]; sys.argv=[sys.argv[0], S, "0"]
exec(open(S+'/glref.py').read())
X=[];Y=[]
for l in open(S+'/gl.u2'):
    a,b,_=l.split(); x=float.fromhex(a)
    if 2.0<x<8.0: X.append(x); Y.append(float.fromhex(b))
X=np.array(X); Y=np.array(Y)
direct=X<3.0
# reduction: t and p
t=X.copy(); steps=np.zeros(len(X),int)
while (t>=3.0).any(): s=t>=3.0; t[s]-=1.0; steps[s]+=1
lnp=np.zeros(len(X)); red=~direct
p=t[red].astype(LD); st_=steps[red]; tr=t[red]
for i in range(1,6):
    s=st_>i; p[s]=sd(p[s]*(tr[s]+i).astype(LD))
print("computing LN for", red.sum(), flush=True)
lnp_red=LN(p.astype(np.float64))
A=k2ref(t); B=k2c9(t)
def full(k):
    out=k.copy()
    out[red]=sd(k[red].astype(LD)+lnp_red.astype(LD)).astype(np.float64)
    return out
FA=full(A); FB=full(B)
T=t-2.0
edges=[0,1e-12,1e-9,1e-7,1e-6,1e-5,1e-4,1e-3,3e-3,5e-3,1e-2,2e-2,5e-2,0.1,0.2,0.5,1.0]
print("t-bin               direct: n ref cand9 | reduced: n ref cand9")
for lo,hi in zip(edges,edges[1:]):
    m=(T>=lo)&(T<hi)
    md=m&direct; mr=m&red
    f=lambda mm,F: (np.mean(F[mm]==Y[mm]) if mm.any() else float('nan'))
    print(f"[{lo:.0e},{hi:.0e})  {md.sum():8d} {f(md,FA):.4f} {f(md,FB):.4f} | {mr.sum():7d} {f(mr,FA):.4f} {f(mr,FB):.4f}")
for cut in [0,1e-6,1e-5,1e-4,1e-3,3e-3,5e-3,1e-2,2e-2,5e-2]:
    F=np.where(T<cut,FB,FA); print(f"cut {cut:.0e}: direct {np.mean(F[direct]==Y[direct]):.5f} reduced {np.mean(F[red]==Y[red]):.5f} all {np.mean(F==Y):.5f}")
