# T-1941 W6: 1-D scan of two coupled coefficients along their function-weak direction, then a local 2-D polish.
# The direction ratio r (ulps of c_a per ulp of c_b) is computed from the function: minimise ||dR|| over [1.5,2].
# Usage: pairscan.py RFILE CHAIN OUT ca cb tmin tmax tstep npts   (ca, cb like N0, D1)
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from lib6 import st, L, horner; from synth import read_chain
rows=[l.split() for l in open(sys.argv[1])]
rng=np.random.default_rng(1); idx=rng.choice(len(rows),min(int(sys.argv[9]),len(rows)),replace=False)
x=np.array([float.fromhex(rows[i][0]) for i in idx]); lo=np.array([float.fromhex(rows[i][2]) for i in idx]); hi=np.array([float.fromhex(rows[i][3]) for i in idx])
N,D=read_chain(sys.argv[2]); N=[v/D[0] for v in N]; D=[v/D[0] for v in D]
def nb(v,k):
    i=np.frombuffer(np.float64(v).tobytes(),np.int64)[0]+(int(k) if v>=0 else -int(k)); return float(np.frombuffer(np.int64(i).tobytes(),np.float64)[0])
def sc(N,D):
    R=st(L(horner(N[::-1],x))/L(horner(D[::-1],x))); return np.mean((R>=lo)&(R<=hi))
ca,cb=sys.argv[4],sys.argv[5]
def get(c): return (N if c[0]=='N' else D),int(c[1:])
# direction from the exact function sensitivities (relative)
xf=np.linspace(1.5,2,50); Nv=np.polyval(N[::-1],xf); Dv=np.polyval(D[::-1],xf)
def sens(c):
    arr,j=get(c); u=np.spacing(abs(arr[j])); return (xf**j*u/Nv) if c[0]=='N' else -(xf**j*u/Dv)
sa,sb=sens(ca),sens(cb); r=-np.dot(sa,sb)/np.dot(sa,sa)   # move b by 1 ulp, a by r ulps
print('direction: %s per %s = %.4f'%(ca,cb,r))
def apply(ta,tb):
    NN=list(N); DD=list(D)
    for c,t in ((ca,ta),(cb,tb)):
        arr=NN if c[0]=='N' else DD; j=int(c[1:]); arr[j]=nb(arr[j],t)
    return NN,DD
best=(sc(N,D),0,0); print('start %.4f'%best[0])
for t in np.arange(float(sys.argv[6]),float(sys.argv[7])+1,float(sys.argv[8])):
    ta=int(round(r*t)); v=sc(*apply(ta,int(t)))
    if v>best[0]: best=(v,ta,int(t)); print('  t %d: (%d,%d) %.4f'%(t,ta,int(t),v),flush=True)
v0,ta0,tb0=best
for da in range(-12,13):
    for db in range(-3,4):
        v=sc(*apply(ta0+da,tb0+db))
        if v>best[0]: best=(v,ta0+da,tb0+db)
print('best (%s %+d, %s %+d): %.4f'%(ca,best[1],cb,best[2],best[0]))
NN,DD=apply(best[1],best[2])
open(sys.argv[3],'w').write('N '+' '.join(float(v).hex() for v in NN)+'\nD '+' '.join(float(v).hex() for v in DD)+'\n')
