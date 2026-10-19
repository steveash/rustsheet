# sweep relative-minimax parents over degree and interval; record Taylor invariants at x=0
import mpmath as mp, sys, json
from remez import remez
from taylor0 import taylor
def f(x): return mp.loggamma(x)/(x-1) if x!=1 else -mp.euler
m,n=int(sys.argv[1]),int(sys.argv[2])
out=open(f'sweep_{m}{n}.txt','a')
for a in sys.argv[3].split(','):
    for b in sys.argv[4].split(','):
        try:
            p,q,E,X=remez(f,mp.mpf(a),mp.mpf(b),m,n,dps=36,grid=700)
            t=taylor(p,q)
            out.write('%d %d %s %s %s %s\n'%(m,n,a,b,mp.nstr(E,4),' '.join(mp.nstr(v,12) for v in t))); out.flush()
        except Exception as e: out.write('%d %d %s %s ERR %s\n'%(m,n,a,b,e)); out.flush()
