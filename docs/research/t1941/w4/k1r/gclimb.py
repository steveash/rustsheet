import numpy as np, mpmath as mp, math, sys, gsk, itertools, time, os
from fpylll import IntegerMatrix, BKZ
sys.path.insert(0,'../wt/docs/research/t1941/w6'); from synth import read_chain
mp.mp.dps=30
N,D=read_chain(sys.argv[1]); C=np.concatenate([N,D]).astype(float); out=sys.argv[2]; WD=float(os.environ.get('WD','2'))
free=[i for i in range(16) if i!=8]; ul=np.array([math.ulp(abs(C[i])) for i in free]); n=len(free)
tg=[mp.mpf(1)+mp.mpf(k)/60 for k in range(61)]; J=[]
for tt in tg:
    Nv=sum(mp.mpf(C[i])*tt**i for i in range(8)); Dv=sum(mp.mpf(C[8+i])*tt**i for i in range(8)); G=Nv/Dv; u=mp.mpf(2)**(mp.floor(mp.log(abs(G),2))-52)
    J.append([float(tt**(i%8)/Dv/u) if i<8 else float(-G*tt**(i%8)/Dv/u) for i in free])
J=np.array(J)*ul; sc=10**6; B=IntegerMatrix(n,n+61)
for j in range(n):
    B[j,j]=sc
    for r in range(61): B[j,n+r]=int(round(100*J[r,j]*sc))
BKZ.reduction(B,BKZ.Param(block_size=12))
V=sorted([(np.max(np.abs(J@v)),v) for v in [np.array([B[j,i] for i in range(n)])//sc for j in range(n)]],key=lambda q:q[0])
moves=[v for f,v in V if f<0.05]; moves=moves+[a+b for a,b in itertools.combinations(moves,2)]+[a-b for a,b in itertools.combinations(moves,2)]
moves+=[np.eye(n,dtype=int)[k]*s for k in range(n) for s in (1,2,4,8)]
moves+=[np.eye(n,dtype=int)[i]*a+np.eye(n,dtype=int)[j]*b for i,j in itertools.combinations(range(n),2) for a in (-1,1) for b in (-2,-1,1,2)]
dE=np.load('k1R10_ev.npz'); x,t=dE['x'][::2],dE['t'][::2]; dip=(x>=1.84)&(x<1.97); xd,td=np.ascontiguousarray(x[dip]),np.ascontiguousarray(t[dip])
dO=np.load('k1R10_od.npz'); xo,to=dO['x'],dO['t']; dipo=(xo>=1.84)&(xo<1.97)
def mk(k): c=C.copy(); c[free]+=k*ul; return c
def obj(k):
    c=mk(k); a=gsk.count(c[:8],c[8:],0,0,x,t)/len(x); b=gsk.count(c[:8],c[8:],0,0,xd,td)/len(xd); return a+WD*b*dip.mean(),a,b
def test(k):
    c=mk(k); return gsk.count(c[:8],c[8:],0,0,xo,to)/len(xo), gsk.count(c[:8],c[8:],0,0,np.ascontiguousarray(xo[dipo]),np.ascontiguousarray(to[dipo]))/dipo.sum()
k=np.zeros(n,dtype=int); cur=obj(k); print('start fit all %.4f dip %.4f | test all %.4f dip %.4f'%(cur[1],cur[2],*test(k)),flush=True); t0=time.time()
for rnd in range(int(os.environ.get('ROUNDS','10'))):
    imp=0
    for mv in moves:
        for s in (1,-1,2,-2,4,-4):
            k2=k+s*mv; v=obj(k2)
            if v[0]>cur[0]: k,cur=k2,v; imp+=1
    print('round %d fit all %.4f dip %.4f | test all %.4f dip %.4f  moves %d %.0fs'%(rnd,cur[1],cur[2],*test(k),imp,time.time()-t0),flush=True)
    if not imp: break
c=mk(k); open(out,'w').write("N "+" ".join(float(v).hex() for v in c[:8])+"\nD "+" ".join(float(v).hex() for v in c[8:])+"\n")
