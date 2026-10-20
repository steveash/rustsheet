import k1lib as K, numpy as np, mpmath as mp, math, sys
from fpylll import IntegerMatrix, LLL, BKZ
mp.mp.dps=40
idx=[('N',i) for i in range(8)]+[('D',i) for i in range(1,8)]
N0,D0=K.N0,K.D0; ul=np.array([math.ulp((N0 if c=='N' else D0)[i]) for c,i in idx])
tg=[mp.mpf(1)+mp.mpf(k)/60 for k in range(61)]; J=[]
for t in tg:
    Nv=sum(mp.mpf(N0[i])*t**i for i in range(8)); Dv=sum(mp.mpf(D0[i])*t**i for i in range(8)); G=Nv/Dv; u=mp.mpf(2)**(mp.floor(mp.log(G,2))-52)
    J.append([float(t**i/Dv/u) if c=='N' else float(-G*t**i/Dv/u) for c,i in idx])
J=np.array(J)*ul
lam=float(sys.argv[1]); scale=10**6
n=len(idx); m=J.shape[0]
B=IntegerMatrix(n,n+m)
for j in range(n):
    B[j,j]=scale
    for r in range(m): B[j,n+r]=int(round(lam*J[r,j]*scale))
BKZ.reduction(B,BKZ.Param(block_size=12))
rows=[]
for j in range(n):
    v=np.array([B[j,i] for i in range(n)])//scale
    rows.append((np.max(np.abs(J@v)),v))
rows.sort(key=lambda t:t[0])
for f,v in rows: print('fmax %.4f  |v|max %6d  '%(f,np.max(np.abs(v)))+' '.join('%s%d:%+d'%(c,i,d) for (c,i),d in zip(idx,v) if d))
