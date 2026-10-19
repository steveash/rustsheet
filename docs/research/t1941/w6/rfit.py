# T-1941 W6: relative least-squares rational fits R ~ N(x)/D(x) (raw x, degrees (n,d)) to R = lnG/((x-1)(x-2)) on
# [1.5,2], via the SK iteration in mpmath; prints coefficients ascending (N then D, D0 normalised to 1).
import sys, mpmath as mp
mp.mp.dps=60
def fit(n,d,lo=1.5,hi=2.0,K=None,iters=8,target=None):
    K=K or 4*(n+d+2)
    xs=[ (lo+hi)/2+(hi-lo)/2*mp.cos(mp.pi*(k+0.5)/K) for k in range(K)]
    f=target or (lambda z: mp.loggamma(z)/((z-1)*(z-2)) if z!=2 else 1-mp.euler)
    ys=[f(x) for x in xs]; w=[mp.mpf(1)]*K; a=b=None
    for it in range(iters):
        A=mp.matrix(K,n+1+d); r=mp.matrix(K,1)
        for i,(x,y) in enumerate(zip(xs,ys)):
            s=w[i]/y   # relative
            for j in range(n+1): A[i,j]=x**j*s
            for j in range(1,d+1): A[i,n+j]=-y*x**j*s
            r[i]=y*s
        sol=mp.lu_solve(A.T*A,A.T*r) if False else mp.qr_solve(A,r)[0]
        a=[sol[j] for j in range(n+1)]; b=[mp.mpf(1)]+[sol[n+j] for j in range(1,d+1)]
        w=[1/abs(mp.polyval(b[::-1],x)) for x in xs]
    err=max(abs(mp.polyval(a[::-1],x)/mp.polyval(b[::-1],x)/y-1) for x,y in zip(xs,ys))
    return a,b,err
if __name__=='__main__':
    for n,d in [tuple(map(int,s.split(','))) for s in sys.argv[1:]]:
        a,b,e=fit(n,d); print('(%d,%d) maxrel %.2e'%(n,d,float(e)))
        print('N',' '.join(float(v).hex() for v in a)); print('D',' '.join(float(v).hex() for v in b))
