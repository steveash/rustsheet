"""Solve the interval [a,b] of a weighted minimax (m,n) parent so that its x=0 Taylor invariants R0,R1 match
Excel's (pinned by W1's low coefficients); report the R2,R3 mismatch as the test."""
import mpmath as mp, sys
from remez2 import remez2 as remez
from taylor0 import taylor
EX=[mp.mpf('-5.4058524839103'),mp.mpf('134.94228237913'),mp.mpf('-5937.3201920600'),mp.mpf('281011.50865')]
def f(x): return mp.loggamma(x)/(x-1) if x!=1 else -mp.euler
W={'rel':None,'absB':lambda x: 1/abs(x-1) if x!=1 else mp.mpf(10)**30,'abs':lambda x: mp.mpf(1),'lgabs':lambda x: 1/abs(x-1) if x!=1 else mp.mpf(10)**30}
m,n,wn=int(sys.argv[1]),int(sys.argv[2]),sys.argv[3]
a,b=mp.mpf(sys.argv[4]),mp.mpf(sys.argv[5])
fix=sys.argv[6] if len(sys.argv)>6 else ''   # 'a' or 'b' to hold one end and solve R0 only
ref=[None]
def inv(a,b):
    r=ref[0]
    if r is not None:
        a0,b0,X=r; r=[a+(x-a0)*(b-a)/(b0-a0) for x in X]
    p,q,E,X=remez(f,a,b,m,n,w=W[wn],dps=40,grid=500,ref=r)
    ref[0]=(a,b,X); t=taylor(p,q); return t,E
h=mp.mpf('1e-4')
for it in range(8):
    t,E=inv(a,b); r0=t[0]-EX[0]; r1=t[1]-EX[1]
    print(it,mp.nstr(a,12),mp.nstr(b,12),'E',mp.nstr(E,4),'dR',[mp.nstr(t[k]/EX[k]-1,4) for k in range(4)],flush=True)
    if fix=='b':
        ta,_=inv(a+h,b); a=a-r0/((ta[0]-t[0])/h)
    elif fix=='a':
        tb,_=inv(a,b+h); b=b-r0/((tb[0]-t[0])/h)
    else:
        ta,_=inv(a+h,b); tb,_=inv(a,b+h)
        J=mp.matrix([[(ta[0]-t[0])/h,(tb[0]-t[0])/h],[(ta[1]-t[1])/h,(tb[1]-t[1])/h]])
        d=mp.lu_solve(J,mp.matrix([-r0,-r1])); a+=d[0]; b+=d[1]
    if abs(r0)<1e-12 and (fix or abs(r1)<1e-9): break
