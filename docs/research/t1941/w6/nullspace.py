# T-1941 W6: exact function-null-space parametrisation for an (n,d) raw-x rational of R on [lo,hi] with D0 = 1.
# Linear rows: (N(x) - R(x) D(x)) / (R(x) Dref(x)) = 0 at Chebyshev nodes, solved/SVD'd in mpmath. A coordinate z_k
# moves the coefficients along null vector k, scaled so that |z_k| = 1 changes stage values by up to 1e-3 (relative).
import numpy as np, mpmath as mp
mp.mp.dps=60
def Rt(x): return mp.loggamma(x)/((x-1)*(x-2)) if x!=2 else 1-mp.euler
class Null:
    def __init__(self,n,d,lo=1.5,hi=2.0,K=80,Dref=None,thr=1e-17):
        self.n,self.d=n,d; xs=[(mp.mpf(lo)+mp.mpf(hi))/2+(mp.mpf(hi)-mp.mpf(lo))/2*mp.cos(mp.pi*(k+mp.mpf(1)/2)/K) for k in range(K)]
        R=[Rt(x) for x in xs]
        if Dref is None: w=[1/r for r in R]
        else: w=[1/(r*mp.polyval([mp.mpf(v) for v in Dref[::-1]],x)) for x,r in zip(xs,R)]
        A=mp.matrix(K,n+1+d); b=mp.matrix(K,1)
        for i,(x,r) in enumerate(zip(xs,R)):
            for j in range(n+1): A[i,j]=x**j*w[i]
            for j in range(1,d+1): A[i,n+j]=-r*x**j*w[i]
            b[i]=r*w[i]
        self.A=A; self.b=b
        sol=mp.qr_solve(A,b)[0]; self.c0=np.array([float(v) for v in sol]); self.c0mp=sol
        U,S,V=mp.svd_r(A)
        self.S=np.array([float(s) for s in S]); self.V=np.array([[float(V[i,j]) for j in range(V.cols)] for i in range(V.rows)])
        self.k=int(np.sum(self.S<thr)); self.Vn=self.V[-self.k:] if self.k else np.zeros((0,n+1+d))
        self.scale=np.ones(self.k)
    def chain(self,z):
        c=self.c0+(self.scale*z)@self.Vn if self.k else self.c0
        return c[:self.n+1], np.concatenate([[1.0],c[self.n+1:]])
    def coords(self,N,D):
        c=np.concatenate([N,D[1:]])/D[0]*np.array([1.0]*(self.n+1)+[1.0]*self.d)
        return (self.Vn@(c-self.c0))/self.scale, float(np.linalg.norm(c-self.c0-(self.Vn.T@(self.Vn@(c-self.c0)))))
    def ferr(self,z):
        N,D=self.chain(z); A=self.A; c=np.concatenate([N,D[1:]])
        r=[sum(A[i,j]*mp.mpf(c[j]) for j in range(A.cols))-self.b[i] for i in range(A.rows)]
        return max(abs(v) for v in r)
