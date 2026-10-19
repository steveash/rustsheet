# T-1941: relative error of a G candidate against true Gamma on [1,2], with its extrema (minimax/equioscillation check).
import mpmath as mp
mp.mp.dps=40
def load(f):
    d={}
    for l in open(f):
        if l.startswith('#'): continue
        p=l.split(); d[p[0]]=[mp.mpf(float.fromhex(v)) for v in p[1:]]
    return d
d=load('docs/research/t1941/g_candidate_w4_AN2.txt')
N,D=d['N'],d['D']
def r(y): return mp.polyval(N[::-1],y)/mp.polyval(D[::-1],y)/mp.gamma(y)-1
ys=[1+mp.mpf(i)/4000 for i in range(4001)]
e=[r(y) for y in ys]
ext=[]
for i in range(1,len(e)-1):
    if (e[i]-e[i-1])*(e[i+1]-e[i])<0: ext.append((float(ys[i]),float(e[i])))
ext=[(1.0,float(e[0]))]+ext+[(2.0,float(e[-1]))]
print(len(ext),'extrema')
for y,v in ext: print('%.4f %+.3e'%(y,v))
