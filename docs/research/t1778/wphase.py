# T-1778: W sawtooth phase at each sweep start (reads the two sweep_delta files); writes phases_all.txt.
import math
K=1.1366973296084131; G=2.0**-53; b=0.4546572618
def fitphase(R,z0):
    zs=[float(r[1]) for r in R]; obs=[(float(r[4])+float(r[5]))/2 for r in R]
    best=None
    for i in range(10000):
        a=i/10000
        pred=[K*G*((a+(z-z0)*b*math.exp(-b*z)/G)-round(a+(z-z0)*b*math.exp(-b*z)/G)) for z in zs]
        c=sum(o-p for o,p in zip(obs,pred))/len(obs)
        err=max(abs(o-p-c) for o,p in zip(obs,pred))
        if best is None or err<best[0]: best=(err,a,c)
    return best
out=[]
small=[l.split() for l in open('sweep_delta_small.txt')]
big=[l.split() for l in open('sweep_delta.txt')]
for name,R in [('tiny',[r for r in small if int(r[0])<64 and int(r[0])!=62 and int(r[3])>=255]),
               ('1e-12',[r for r in small if 64<=int(r[0])<112 and int(r[3])>=255]),
               ('2e-10',[r for r in small if 112<=int(r[0])<160 and int(r[3])>=255])]+[('zs%d'%s,[r for r in big if int(r[0])//24==s and int(r[3])>=250]) for s in range(5)]:
    z0=float(R[0][1]); e,a,c=fitphase(R,z0)
    print(name,'z0 %.17g phase %.4f err %.2e n %d'%(z0,a,e,len(R)))
    out.append(f"{name} {z0!r} {a}\n")
open('phases_all.txt','w').writelines(out)
