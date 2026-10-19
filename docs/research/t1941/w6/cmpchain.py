# T-1941 W6: compare a pole-offset solution against a truth chain: relative differences of coefficients and of the
# stage values at x = 1.5, 1.75, 2. Usage: cmpchain.py truthchain n lo hi "offsets..."
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from poleobj import dpoly, npoly; from rfit import fit; from synth import read_chain; from stagelaws import stage_F
mp.mp.dps=80
N,D=read_chain(sys.argv[1]); N=np.array(N); D=np.array(D); s=1/D[0]; N*=s; D*=s
n=int(sys.argv[2]); a,b,e=fit(n,n,mp.mpf(sys.argv[3]),mp.mpf(sys.argv[4]),iters=10)
p0=np.array(sorted([float(mp.re(r)) for r in mp.polyroots(b[::-1],maxsteps=200,extraprec=300)])); z=np.array([float(v) for v in sys.argv[5].split()])
D2=dpoly(-np.exp(np.log(-p0)+z)); N2=npoly(D2,n)
print('coef rel diff N:',' '.join('%.1e'%v for v in np.abs(N2/N-1))); print('coef rel diff D:',' '.join('%.1e'%v for v in np.abs(D2/D-1)))
x=np.array([1.5,1.75,2.0])
for ch,c1,c2 in (('N',N,N2),('D',D,D2)):
    A=stage_F(c1,x); B=stage_F(c2,x)
    print(ch,'stage rel diff @1.5/1.75/2:',' '.join('%s:%.0e'%(k,np.max(np.abs(B[k][1]/A[k][1]-1))) for k in A if k[0]=='S'))
