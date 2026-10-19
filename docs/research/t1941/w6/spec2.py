# T-1941 W6: spectrum of Excel's B residual on (1.5,2] runs with coefficient-free predicted lines:
#  fB  final product t*A (sawtooth law, = lnG'*h/ulp(B));
#  fmR product m*R into a stored A, R = lnG/((x-1)(x-2)), law (R + m R')*h/ulp(A);
#  fA  rounding of a quotient A = lnG/(x-1) (A'*h/ulp(A));  fS  quotient S = lnG/(x-2).
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import load
mp.mp.dps=40
X,B,C=load(sys.argv[1]); cases=sys.argv[2:]
def alias(f): f=float(f)%1.0; return min(f,1-f)
def ulp(v): return mp.mpf(np.spacing(abs(float(v))))
for cs in cases:
    s=(C==cs); x=X[s]; b=B[s]; n=1<<int(np.log2(len(x))); x=x[:n]; b=b[:n]; h=mp.mpf(float(x[1]-x[0]))
    x0=mp.mpf(float(x[0])); g=[mp.diff(mp.loggamma,x0,k)/mp.factorial(k) for k in range(4)]
    r=np.array([float((mp.mpf(float(bi))-(g[0]+d*(g[1]+d*(g[2]+d*g[3]))))/ulp(bi)) for bi,d in ((bi,mp.mpf(float(xi))-x0) for xi,bi in zip(x,b))])
    F=np.abs(np.fft.rfft(r-r.mean()))/n*2; fr=np.fft.rfftfreq(n)
    lg=lambda z: mp.loggamma(z)
    t=x0-1; m=x0-2; A=g[0]/t; R=g[0]/(t*m); S=g[0]/m
    Ad=mp.diff(lambda z: lg(z)/(z-1),x0); Rd=mp.diff(lambda z: lg(z)/((z-1)*(z-2)),x0); Sd=mp.diff(lambda z: lg(z)/(z-2),x0)
    pr=dict(fB=alias(g[1]*h/ulp(g[0])),fmR=alias((R+m*Rd)*h/ulp(A)),fA=alias(Ad*h/ulp(A)),fS=alias(Sd*h/ulp(S)),fR=alias(Rd*h/ulp(R)))
    top=np.argsort(F)[::-1][:14]
    print('%s x0=%.5f std %.2f  A=%.4f R=%.4f | '%(cs,float(x0),r.std(),float(A),float(R))+' '.join('%s %.5f(%.3f)'%(k,v,F[int(round(v*n))]) for k,v in pr.items()))
    print('   peaks: '+' '.join('%.5f(%.3f)'%(fr[j],F[j]) for j in top))
