# T-1941 W6: line spectrum of Excel's inner R residual (R - Rtrue)/ulp(R) on long consecutive runs (R midpoint of its
# candidate set). Predicted coefficient-free lines: fR = R'h/ulp(R) (rounding of a smooth R, e.g. fl(c + small P));
# fP = (Q + mQ')h/ulp(P) with Q = (R - (1-gamma))/m (Cody form R = fl(c + fl(m*Q))).
import sys, numpy as np, mpmath as mp, collections
mp.mp.dps=40
rows=collections.defaultdict(list)
for l in open(sys.argv[1]):
    p=l.split(); rows[p[5]].append((float.fromhex(p[0]),float.fromhex(p[2]),float.fromhex(p[3]),int(p[4])))
def alias(f): f=float(f)%1.0; return min(f,1-f)
Rf=lambda z: mp.loggamma(z)/((z-1)*(z-2))
for cs in sys.argv[2:]:
    v=np.array(rows[cs]); x=v[:,0]; n=1<<int(np.log2(len(x))); v=v[:n]; x=x[:n]; Rm=(v[:,1]+v[:,2])/2; u=np.spacing(v[:,1])
    x0=mp.mpf(x[0]); g=[mp.diff(Rf,x0,k)/mp.factorial(k) for k in range(3)]
    e=np.array([float((mp.mpf(Rm[i])-(g[0]+(mp.mpf(x[i])-x0)*(g[1]+(mp.mpf(x[i])-x0)*g[2])))/mp.mpf(u[i])) for i in range(n)])
    F=np.abs(np.fft.rfft(e-e.mean()))/n*2; fr=np.fft.rfftfreq(n); h=mp.mpf(x[1]-x[0])
    c=mp.mpf(float(1-mp.euler)); m=x0-2; Q=(g[0]-c)/m; Qd=mp.diff(lambda z:(Rf(z)-c)/(z-2),x0); P=m*Q
    fR=alias(g[1]*h/mp.mpf(np.spacing(float(g[0])))); fP=alias((Q+m*Qd)*h/mp.mpf(np.spacing(abs(float(P)))))
    top=np.argsort(F)[::-1][:16]
    print('%s x0=%.5f n=%d sd %.2f  fR %.5f(%.3f) fP %.5f(%.3f)'%(cs,float(x0),n,e.std(),fR,F[int(round(fR*n))],fP,F[int(round(fP*n))]))
    print('   '+' '.join('%.5f(%.3f)'%(fr[j],F[j]) for j in top))
