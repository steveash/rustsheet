# compute and cache parent rationals (relative minimax of lgamma(x)/(x-1)) for several intervals / degrees
import mpmath as mp, pickle, sys, os
from remez import remez
def f(x): return mp.loggamma(x)/(x-1) if x!=1 else -mp.euler
out={}
if os.path.exists('parents.pkl'): out=pickle.load(open('parents.pkl','rb'))
jobs=[(8,8,'0.7','1.5'),(7,7,'0.7','1.5'),(6,6,'0.7','1.5'),(9,9,'0.7','1.5'),(8,9,'0.7','1.5'),(9,8,'0.7','1.5')]
for m,n,a,b in jobs:
    k=(m,n,a,b,'rel')
    if k in out: continue
    p,q,E,X=remez(f,mp.mpf(a),mp.mpf(b),m,n,dps=50,grid=1500)
    out[k]=([str(v) for v in p],[str(v) for v in q],str(E)); print(k,mp.nstr(E,5),flush=True)
    pickle.dump(out,open('parents.pkl','wb'))
