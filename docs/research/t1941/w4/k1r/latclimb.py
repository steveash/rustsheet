import k1lib as K, numpy as np, mpmath as mp, math, sys, pickle, itertools
from fpylll import IntegerMatrix, BKZ
mp.mp.dps=40
idx=[('N',i) for i in range(8)]+[('D',i) for i in range(1,8)]
def jac(N0,D0):
    ul=np.array([math.ulp((N0 if c=='N' else D0)[i]) for c,i in idx]); tg=[mp.mpf(1)+mp.mpf(k)/60 for k in range(61)]; J=[]
    for t in tg:
        Nv=sum(mp.mpf(N0[i])*t**i for i in range(8)); Dv=sum(mp.mpf(D0[i])*t**i for i in range(8)); G=Nv/Dv; u=mp.mpf(2)**(mp.floor(mp.log(G,2))-52)
        J.append([float(t**i/Dv/u) if c=='N' else float(-G*t**i/Dv/u) for c,i in idx])
    return np.array(J)*ul
def basis(J,lam=100):
    n=len(idx); m=J.shape[0]; sc=10**6; B=IntegerMatrix(n,n+m)
    for j in range(n):
        B[j,j]=sc
        for r in range(m): B[j,n+r]=int(round(lam*J[r,j]*sc))
    BKZ.reduction(B,BKZ.Param(block_size=12))
    V=np.array([[B[j,i]//sc for i in range(n)] for j in range(n)]); f=np.max(np.abs(J@V.T),axis=0); o=np.argsort(f); return V[o],f[o]
def apply(N,D,v):
    N=N.copy(); D=D.copy()
    for (c,i),d in zip(idx,v):
        if d: A=N if c=='N' else D; A[i]=A[i]+d*math.ulp(A[i])
    return N,D
RUNS=K.runs(sys.argv[1].split(','))
Y=np.concatenate([r[1] for r in RUNS]); T=np.concatenate([r[2] for r in RUNS])
def score(N,D): return K.gsimpy.Sim(N,D).count(Y,T)
N,D=K.N0.copy(),K.D0.copy(); J=jac(N,D); V,f=basis(J)
cur=score(N,D); print('start exact %d / %d (%.4f)'%(cur,len(Y),cur/len(Y)),flush=True)
FT=float(sys.argv[2]); tot=np.zeros(len(idx),int)
for it in range(int(sys.argv[3])):
    best=(cur,None)
    for j in range(len(V)):
        if f[j]>FT: continue
        for s in (1,-1,2,-2):
            v=tot+s*V[j]
            if np.max(np.abs(J@v))>FT: continue
            sc=score(*apply(K.N0,K.D0,v))
            if sc>best[0]: best=(sc,v)
    if best[1] is None: print('no improving lattice move; stop'); break
    tot=best[1]; cur=best[0]
    print('it %d exact %d (%.4f)  move %s  fchange %.4f'%(it,cur,cur/len(Y),' '.join('%s%d:%+d'%(c,i,d) for (c,i),d in zip(idx,tot) if d),np.max(np.abs(J@tot))),flush=True)
pickle.dump(tot,open('latclimb_move.pkl','wb'))
