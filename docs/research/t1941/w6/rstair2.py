# T-1941 W6: R staircase per sweep segment (512 consecutive doubles), Rtrue by local Taylor (fast).
import sys, numpy as np, mpmath as mp, collections
mp.mp.dps=30
Rf=lambda z: mp.loggamma(z)/((z-1)*(z-2))
seg=collections.defaultdict(list)
for l in open(sys.argv[1]):
    p=l.split()
    if not p[5].startswith('kmyst-gam-q4'): continue
    x=float.fromhex(p[0]); seg[(p[5],round(x,4))].append((x,float.fromhex(p[2]),float.fromhex(p[3]),int(p[4])))
out=[]
for k,v in seg.items():
    v=np.array(v)
    if len(v)<300: continue
    x=v[:,0]; Rm=(v[:,1]+v[:,2])/2; x0=mp.mpf(x[0]); g=[mp.diff(Rf,x0,j)/mp.factorial(j) for j in range(3)]
    e=np.array([float((mp.mpf(Rm[i])-(g[0]+(mp.mpf(x[i])-x0)*(g[1]+(mp.mpf(x[i])-x0)*g[2])))/mp.mpf(np.spacing(v[i,1]))) for i in range(len(x))])
    out.append((x[0],len(x),e.mean(),e.std()/np.sqrt(len(e)),e.std()))
out.sort()
np.savetxt(sys.argv[2],np.array(out),fmt='%.6f')
for o in out: print('%.5f n=%d mean %+.2f se %.2f sd %.2f'%o)
