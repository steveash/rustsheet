# T-1941 W6: synthetic data from an m-variable chain (m = x-2 exact; R = fl(N(m)/D(m)), Horner in m, every op
# stored), written like synth.py. Chain file holds coefficients ascending in m. Also builds a (9,9) relative-LS fit
# in m when called with "fit lo hi out".
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import st, L, horner; from synth import read_chain
if sys.argv[1]=='fit':
    from rfit import fit
    mp.mp.dps=80
    lo,hi=float(sys.argv[2]),float(sys.argv[3])
    f=lambda z: mp.loggamma(z+2)/((z+1)*z) if z!=0 else 1-mp.euler
    a,b,e=fit(9,9,mp.mpf(lo)-2,mp.mpf(hi)-2,target=f,iters=10); s=1/b[0]
    open(sys.argv[4],'w').write('# m-variable (9,9) relative-LS of R on [%g,%g], D0=1, err %.1e\nN '%(lo,hi,float(e))+' '.join(float(v*s).hex() for v in a)+'\nD '+' '.join(float(v*s).hex() for v in b)+'\n')
    sys.exit()
N,D=read_chain(sys.argv[1]); rows=[l.split() for l in open(sys.argv[2]) if l.split()[5].startswith(sys.argv[4])]
x=np.array([float.fromhex(r[0]) for r in rows]); m=x-2.0
R=st(L(horner(N[::-1],m))/L(horner(D[::-1],m))); B=st(L(x-1.0)*L(st(L(m)*L(R))))
t=x-1.0; R0=B/(t*m); H=[]
def nb(v,k):
    for _ in range(abs(k)): v=np.nextafter(v,np.inf if k>0 else -np.inf)
    return v
for k in range(-5,6):
    Rk=nb(R0,k); H.append(np.where(st(L(t)*L(st(L(m)*L(Rk))))==B,Rk,np.nan))
H=np.array(H); lo=np.nanmin(H,0); hi=np.nanmax(H,0); n=np.sum(~np.isnan(H),0)
with open(sys.argv[3],'w') as o:
    for i,r in enumerate(rows): o.write('%s %s %s %s %d %s\n'%(r[0],B[i].hex(),lo[i].hex(),hi[i].hex(),n[i],r[5]))
