# T-1941 W6: is R a plain polynomial? Chebyshev interpolant of R = lnG/((x-1)(x-2)) on [1.5,2] (degree K, mpmath),
# evaluated in double x87-stored arithmetic as (a) Clenshaw in z = 4x-7 (z stored), (b) Horner in z,
# (c) Horner in x-1.75, (d) Horner in x-1.5, (e) Horner in raw x. Score: exact-simulation spectral coherence against
# Excel's R residuals on sweep segments (simobj cache) and the fraction of R values matched exactly.
import sys, numpy as np, mpmath as mp, pickle
sys.path.insert(0,sys.path[0]); from lib6 import st, L
mp.mp.dps=60
X,T,E=pickle.load(open(sys.argv[1],'rb')); Ez=(E-E.mean(1,keepdims=True))/E.std(1,keepdims=True); Xa=X.reshape(-1)
def Rt(x): return mp.loggamma(x)/((x-1)*(x-2)) if x!=2 else 1-mp.euler
def cheb(K):
    nodes=[mp.cos(mp.pi*(k+mp.mpf(1)/2)/(K+1)) for k in range(K+1)]
    f=[Rt((z+7)/4) for z in nodes]
    return [2*sum(f[k]*mp.cos(j*mp.pi*(k+mp.mpf(1)/2)/(K+1)) for k in range(K+1))/(K+1) for j in range(K+1)]
def cheb_to_mono(a,K):   # in z
    T=[[mp.mpf(1)],[mp.mpf(0),mp.mpf(1)]]
    for k in range(2,K+1):
        t=[mp.mpf(0)]+[2*v for v in T[k-1]]; 
        for i,v in enumerate(T[k-2]): t[i]-=v
        T.append(t)
    c=[mp.mpf(0)]*(K+1)
    for j in range(K+1):
        w=a[j]/2 if j==0 else a[j]
        for i,v in enumerate(T[j]): c[i]+=w*v
    return c
def shift(c,s,sc):  # poly in z -> poly in y where z = sc*y + s
    out=[mp.mpf(0)]*len(c)
    for i,ci in enumerate(c):
        for k in range(i+1): out[k]+=ci*mp.binomial(i,k)*sc**k*s**(i-k)
    return out
def hornerv(c,v):
    s=np.full_like(v,float(c[-1]))
    for ci in c[-2::-1]: s=st(L(st(L(s)*L(v)))+L(float(ci)))
    return s
def clenshaw(a,z):
    b1=np.zeros_like(z); b2=np.zeros_like(z); z2=st(L(2.0)*L(z))
    for ak in a[:0:-1]:
        b0=st(L(st(L(st(L(z2)*L(b1)))-L(b2)))+L(float(ak))); b2=b1; b1=b0
    return st(L(st(L(st(L(z)*L(b1)))-L(b2)))+L(float(a[0])/1.0*0.5))
def score(R):
    R=R.reshape(X.shape); out=[]; ex=0; n=0
    for i,(hi,lo,u) in enumerate(T):
        e=((R[i]-hi)-lo)/u; F=np.abs(np.fft.rfft(e-e.mean()))[3:257]; F=(F-F.mean())/F.std(); out.append(np.mean(F*Ez[i]))
    return np.mean(out)
z=st(L(4.0)*L(Xa)-L(7.0))
for K in [int(v) for v in sys.argv[2].split(',')]:
    a=cheb(K); c=cheb_to_mono(a,K)
    res={'clenshaw z':clenshaw(a,z),'horner z':hornerv(c,z),
         'horner x-1.75':hornerv(shift(c,0,4),st(L(Xa)-L(1.75))),'horner x-1.5':hornerv(shift(c,-1,4),st(L(Xa)-L(1.5))),
         'horner x':hornerv(shift(c,-7,4),Xa)}
    print('K=%d  |a_K|=%.1e  '%(K,float(abs(a[K])))+'  '.join('%s %.3f'%(k,score(v)) for k,v in res.items()),flush=True)
