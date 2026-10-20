import k1lib as K, numpy as np, mpmath as mp, math, sys
mp.mp.dps=40
fixed=dict((t.split(':')[0],int(t.split(':')[1])) for t in sys.argv[1].split(','))
free=sys.argv[2].split(',')
N0,D0=K.N0,K.D0
def G(N,D,t): return sum(mp.mpf(N[i])*t**i for i in range(8))/sum(mp.mpf(D[i])*t**i for i in range(8))
N1,D1=N0.copy(),D0.copy()
for k,d in fixed.items():
    A=N1 if k[0]=='N' else D1; A[int(k[1])]+=d*math.ulp(A[int(k[1])])
ts=[mp.mpf(1)+mp.mpf(k)/200 for k in range(201)]; A=[];b=[]
for t in ts:
    g0=G(N0,D0,t); u=mp.mpf(2)**(mp.floor(mp.log(g0,2))-52); g1=G(N1,D1,t); Dv=sum(mp.mpf(D1[i])*t**i for i in range(8))
    b.append(float((g0-g1)/u)); A.append([float((t**int(k[1])/Dv if k[0]=='N' else -g1*t**int(k[1])/Dv)*math.ulp((N1 if k[0]=='N' else D1)[int(k[1])])/u) for k in free])
A=np.array(A); b=np.array(b); sol,*_=np.linalg.lstsq(A,b,rcond=None); isol=np.round(sol).astype(int)
print('compensation:',' '.join('%s:%+d'%(k,v) for k,v in zip(free,isol)),' residual max %.3f G-ulps'%np.max(np.abs(A@isol-b)))
for k,v in zip(free,isol):
    C=N1 if k[0]=='N' else D1; C[int(k[1])]+=v*math.ulp(C[int(k[1])])
x,g,c=K.lib2.g_rows(); m=(x>=1)&(x<2); x,g=x[m],g[m]; x,ui=np.unique(x,return_index=True); g=g[ui]
s0=K.gsimpy.Sim(N0,D0); s1=K.gsimpy.Sim(N1,D1)
ok0=s0.G(x)==g; ok1=s1.G(x)==g; ten=np.floor((x-1)*10).astype(int)
print('AN2 ',' '.join('%.3f'%ok0[ten==t].mean() for t in range(10)),' all %.4f'%ok0.mean())
print('test',' '.join('%.3f'%ok1[ten==t].mean() for t in range(10)),' all %.4f'%ok1.mean())
