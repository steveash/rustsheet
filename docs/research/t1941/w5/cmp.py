import mpmath as mp, pickle, numpy as np
from sc import candidate
mp.mp.dps=50
P=pickle.load(open('parents.pkl','rb'))
N=np.array(candidate('N3'));D=np.array(candidate('D3'))
def get(k): p,q,E=P[k]; return [mp.mpf(v) for v in p],[mp.mpf(v) for v in q]
for k in P:
    p,q=get(k); 
    print(k,'N roots',np.sort_complex(np.roots([float(v) for v in p[::-1]]))[-3:],'D roots',np.sort_complex(np.roots([float(v) for v in q[::-1]]))[-3:])
p,q=get((8,8,'0.7','1.5','rel'))
s=mp.mpf(2)**-13
def mul(c,F):
    out=[mp.mpf(0)]*(len(c)+len(F)-1)
    for i,a in enumerate(c):
        for j,f in enumerate(F): out[i+j]+=a*f
    return out
# best alpha from top coefficients: N9 = alpha*p8*s ; D9 = alpha*q8*s
print('alpha from N9', N[9]/float(p[8]*s), 'from D9', D[9]/float(q[8]*s))
for al in [N[9]/float(p[8]*s)]:
    n=[float(v*s) for v in mul(p,[1,mp.mpf(al)])]; d=[float(v*s) for v in mul(q,[1,mp.mpf(al)])]
    print('rel diff N', ['%.1e'%(a/b-1) for a,b in zip(n,N)]); print('rel diff D', ['%.1e'%(a/b-1) for a,b in zip(d,D)])
