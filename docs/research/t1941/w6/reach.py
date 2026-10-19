# T-1941 W6: model-free final-op test. For B = fl(f * S) with f an exactly known double factor and S any stored double,
# not every double B is reachable when |f|*ulp(S) > ulp(B). Report, per run, the fraction of Excel's B values that
# are reachable for each candidate factor f(x), and the fraction expected for random B (fraction of reachable doubles
# near B), so a structure with a sharp excess is the final op.
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from lib6 import *
X,B,C=load(sys.argv[1],float(sys.argv[2]),float(sys.argv[3]))
def reachable(f,b):
    q=b/f; ok=np.zeros(len(b),bool)
    for d in range(-3,4):
        Sv=q.copy()
        Sv=np.where(d>0,q,q)
        for _ in range(abs(d)): Sv=np.nextafter(Sv,np.inf if d>0 else -np.inf)
        ok|=(st(L(f)*L(Sv))==b)
    return ok
facs={'x-1':lambda x: x-1.0,'x-2':lambda x: x-2.0,'x':lambda x:x,'(x-1)(x-2)st':lambda x: st(L(x-1.0)*L(x-2.0))}
# reference: neighbours of B (B+-1..3 ulps) under the same factor give the base rate of reachable doubles
for lo in np.arange(float(sys.argv[2]),float(sys.argv[3]),float(sys.argv[4])):
    s=(X>=lo)&(X<lo+float(sys.argv[4]))
    if s.sum()<100: continue
    x=X[s]; b=B[s]; out=[]
    for k,f in facs.items():
        fx=f(x); r=reachable(fx,b).mean()
        base=np.mean([reachable(fx,np.nextafter(b,np.inf) if j==1 else np.nextafter(np.nextafter(b,-np.inf),-np.inf)).mean() for j in (1,2)])
        out.append('%s %.3f/%.3f'%(k,r,base))
    print('%.3f %6d  '%(lo,s.sum())+'  '.join(out))
