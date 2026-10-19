# T-1941 W6: line spectrum of (Excel GAMMALN.PRECISE - exact lnGamma)/ulp on consecutive-double runs, with the
# coefficient-free predicted frequencies of the final-quotient lines: A = lnG/(x-1) (near-1 kernel), S = lnG/(x-2)
# ((x-2) kernel), and B itself. Frequencies are aliased to [0,0.5] cycles per step.
import sys, json, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import load
mp.mp.dps=40
X,B,C=load(sys.argv[1]); cases=sys.argv[2:]
def alias(f): f=f%1.0; return min(f,1-f)
def ulp(v): return mp.mpf(np.spacing(abs(float(v))))
for cs in cases:
    s=(C==cs); x=X[s]; b=B[s]
    if len(x)<1024: continue
    # take the longest consecutive-double stretch
    d=np.diff(x); h=np.median(d); ok=np.isclose(d,h,rtol=0,atol=0)
    i0=0; best=(0,0); 
    for i in range(len(d)+1):
        if i==len(d) or not ok[i]:
            if i-i0>best[1]-best[0]: best=(i0,i)
            i0=i+1
    x=x[best[0]:best[1]+1]; b=b[best[0]:best[1]+1]; n=len(x); n=1<<int(np.log2(n)); x=x[:n]; b=b[:n]
    x0=mp.mpf(float(x[0])); H=mp.mpf(float(h))
    g=[mp.diff(mp.loggamma,x0,k)/mp.factorial(k) for k in range(4)]
    k=np.arange(n); dx=[mp.mpf(int(i))*H for i in k]
    ub=np.spacing(np.abs(b))
    r=np.array([float((mp.mpf(float(b[i]))-(g[0]+dx[i]*(g[1]+dx[i]*(g[2]+dx[i]*g[3]))))/mp.mpf(float(ub[i]))) for i in range(n)])
    rr=r-r.mean(); F=np.abs(np.fft.rfft(rr))/n*2; fr=np.fft.rfftfreq(n)
    top=np.argsort(F)[::-1][:12]
    lg=g[0]; d1=g[1]
    A=lg/(x0-1); Ad=(d1*(x0-1)-lg)/(x0-1)**2
    S=lg/(x0-2); Sd=(d1*(x0-2)-lg)/(x0-2)**2
    fA=alias(float(Ad*H/ulp(A))); fS=alias(float(Sd*H/ulp(S))); fB=alias(float(d1*H/ulp(lg)))
    print('%s x0=%.6f n=%d mean %+.2f std %.2f | pred fA %.6f fS %.6f fB %.6f'%(cs,float(x0),n,r.mean(),r.std(),fA,fS,fB))
    print('   peaks: '+' '.join('%.6f(%.3f)'%(fr[j],F[j]) for j in sorted(top,key=lambda j:-F[j])))
