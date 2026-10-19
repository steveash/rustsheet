# T-1941 W6: positive control for excluding R = fl(Y + small) (Boost-like Y + R layout with R stored). Simulate
# R = fl(Y + fl(Rtrue - Y)) for a few Y and measure the fR = R'h/ulp(R) line amplitude, vs Excel's (rspec.py).
import sys, numpy as np, mpmath as mp
mp.mp.dps=40
L=np.longdouble
Rf=lambda z: mp.loggamma(z)/((z-1)*(z-2))
def st(v): return np.asarray(v,dtype=L).astype(np.float64)
for x0 in (1.62346,1.66,1.73207,1.81,1.97):
    n=8192; x=np.array([x0+i*2.0**-52 for i in range(n)]); X0=mp.mpf(x[0]); g=[mp.diff(Rf,X0,k)/mp.factorial(k) for k in range(3)]
    tr=[g[0]+(mp.mpf(xi)-X0)*(g[1]+(mp.mpf(xi)-X0)*g[2]) for xi in x]
    h=mp.mpf(2)**-52; fR=float(g[1]*h/mp.mpf(np.spacing(float(g[0]))))%1; fR=min(fR,1-fR)
    out=[]
    for Y in (0.452017307281494140625,0.4228,0.5):
        q=np.array([float(v-Y) for v in tr]); qr=np.array([float(v-Y-mp.mpf(float(v-Y))) for v in tr])
        R=st(L(Y)+L(q))   # small part stored (its own error ~ulp(q) << ulp(R))
        e=np.array([float((mp.mpf(R[i])-tr[i])/mp.mpf(np.spacing(R[i]))) for i in range(n)])
        F=np.abs(np.fft.rfft(e-e.mean()))/n*2; j=int(round(fR*n))
        out.append('Y=%.3f fR-line %.3f (max %.3f), sd %.2f'%(Y,F[j-2:j+3].max(),F[3:].max(),e.std()))
    print('%.5f fR %.5f: '%(x0,fR)+' | '.join(out))
