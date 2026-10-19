# T-1941 W6: per sweep segment (512 consecutive doubles) spectral peaks of Excel's R residual (zero-padded x8,
# Hann window). Output "x0 freq amp" for the top peaks; saved for line tracking.
import sys, numpy as np, mpmath as mp, collections
mp.mp.dps=30
Rf=lambda z: mp.loggamma(z)/((z-1)*(z-2))
seg=collections.defaultdict(list)
for l in open(sys.argv[1]):
    p=l.split()
    if not p[5].startswith('kmyst-gam-q4') or p[5] in ('kmyst-gam-q409','kmyst-gam-q410'): continue
    x=float.fromhex(p[0]); seg[(p[5],round(x,4))].append((x,float.fromhex(p[2]),float.fromhex(p[3])))
out=open(sys.argv[2],'w'); K=int(sys.argv[3])
for k,v in sorted(seg.items(),key=lambda kv:kv[1][0][0]):
    v=np.array(v)
    for s0 in range(0,len(v)-511,512):
        w=v[s0:s0+512]; x=w[:,0]; Rm=(w[:,1]+w[:,2])/2; x0=mp.mpf(x[0]); g=[mp.diff(Rf,x0,j)/mp.factorial(j) for j in range(3)]
        e=np.array([float((mp.mpf(Rm[i])-(g[0]+(mp.mpf(x[i])-x0)*(g[1]+(mp.mpf(x[i])-x0)*g[2])))/mp.mpf(np.spacing(w[i,1]))) for i in range(512)])
        e=(e-e.mean())*np.hanning(512); F=np.abs(np.fft.rfft(e,4096))/512*4; fr=np.fft.rfftfreq(4096)
        loc=[j for j in range(3,len(F)-1) if F[j]>=F[j-1] and F[j]>=F[j+1] and fr[j]>0.004]
        loc.sort(key=lambda j:-F[j])
        for j in loc[:K]: out.write('%.6f %.5f %.3f\n'%(x[0],fr[j],F[j]))
        if s0>=512*3: break
