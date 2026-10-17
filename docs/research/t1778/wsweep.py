# T-1778: per-run lo offsets (W sawtooth) on the zsweep3 runs; writes sweep_delta.txt.
import math, subprocess
from fractions import Fraction as F
A=1.1170334248975802; U=math.ulp(A)
h=[0,0.14068131264230,0.0448191707,0.00180525,0.00018]
Z={}
for l in open('zs3.txt'):
    p=l.split(); r=int(p[0])
    if r not in Z: x=float.fromhex(p[1]); Z[r]=x*x
out=open('sweep_delta.txt','w')
for r in range(144):
    z=Z[r]; Hu=sum(h[k]*z**k for k in range(1,5))/U; m0=round(Hu)
    best=None
    for m in (m0-1,m0,m0+1):
        o=subprocess.run(['./covd','zs3.txt',str(r),'5','0']+[repr(v) for v in h[1:]]+[str(m)],capture_output=True,text=True,env={'KW':'0','BETA':'0.4546572618','WF':'0'}).stdout.split()
        c=int(o[1]); 
        if best is None or c>best[1]: best=(m,c,float(o[2]),float(o[3]))
    out.write(f"{r} {z!r} {best[0]} {best[1]} {best[2]!r} {best[3]!r} {Hu-best[0]:.3f}\n"); out.flush()
