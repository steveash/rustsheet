# T-1941 W6: which fit interval/degree reproduces Excel's known kernel poles? Relative-LS (SK) fits of
# K3 target lnG/(x-1) and K2 target lnG/(x-2) over intervals [lo,hi]; distance = rms log-ratio of the poles.
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from rfit import fit
mp.mp.dps=80
tA=lambda z: mp.loggamma(z)/(z-1) if z!=1 else -mp.euler
tS=lambda z: mp.loggamma(z)/(z-2) if z!=2 else 1-mp.euler
EX={'K3':[-27.0,-7.45,-3.108,-1.476,-0.723,-0.320,-0.1165,-0.0209],'K2':[-20.06,-3.526,-1.201,-0.540,-0.280,-0.132,-0.0369]}
which=sys.argv[1]; f=tA if which=='K3' else tS; ex=np.array(EX[which])
res=[]
for n,dd in [tuple(map(int,s.split(','))) for s in sys.argv[2].split(';')]:
    for lo in [float(v) for v in sys.argv[3].split(',')]:
        for hi in [float(v) for v in sys.argv[4].split(',')]:
            if hi<=lo: continue
            try: a,b,e=fit(n,dd,mp.mpf(lo),mp.mpf(hi),target=f,iters=10)
            except Exception: continue
            r=sorted([float(mp.re(z)) for z in mp.polyroots(b[::-1],maxsteps=300,extraprec=300)])
            rr=np.array(r[-len(ex):]) if len(r)>=len(ex) else None
            if rr is None or np.any(rr>=0): continue
            dist=np.sqrt(np.mean(np.log(rr/ex)**2)); res.append((dist,n,dd,lo,hi,float(e),r))
res.sort(key=lambda t:t[0])
for t in res[:8]: print('%.3f (%d,%d) [%.2f,%.2f] err %.1e poles %s'%(t[0],t[1],t[2],t[3],t[4],t[5],' '.join('%.4g'%p for p in t[6])))
