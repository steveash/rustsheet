# T-1941 W6: simulate candidate (1.5,2] kernels B = fl(t*fl(m*fl(N/D))) (raw-x Horner, every op stored) from fitted
# rationals and score them on Excel's long runs: R-level |FFT| correlation (structure) and B exact rate.
import sys, numpy as np, mpmath as mp, collections
sys.path.insert(0,sys.path[0]); from lib6 import *; from rfit import fit
mp.mp.dps=40
rows=collections.defaultdict(list)
for l in open(sys.argv[1]):
    p=l.split(); rows[p[5]].append((float.fromhex(p[0]),float.fromhex(p[1]),float.fromhex(p[2]),float.fromhex(p[3])))
runs=['kmyst-gam-r711:B','kmyst-gam-p201:B','kmyst-gam-r712:B','kmyst-gam-p202:B','kmyst-gam-r111:B','kmyst-gam-p203:B','kmyst-gam-r713:B','kmyst-gam-r714:B','kmyst-gam-p204:B']
Rf=lambda z: mp.loggamma(z)/((z-1)*(z-2))
cache={}
for cs in runs:
    v=np.array(rows[cs]); n=1<<int(np.log2(len(v))); v=v[:n]; x=v[:,0]; Rm=(v[:,2]+v[:,3])/2; u=np.spacing(v[:,2])
    x0=mp.mpf(x[0]); g=[mp.diff(Rf,x0,k)/mp.factorial(k) for k in range(3)]
    tr=np.array([float(g[0]+(mp.mpf(xi)-x0)*(g[1]+(mp.mpf(xi)-x0)*g[2])) for xi in x])
    trl=np.array([float(g[0]+(mp.mpf(xi)-x0)*(g[1]+(mp.mpf(xi)-x0)*g[2])-mp.mpf(float(g[0]+(mp.mpf(xi)-x0)*(g[1]+(mp.mpf(xi)-x0)*g[2])))) for xi in x])
    cache[cs]=(x,v[:,1],Rm,u,tr,trl)
def fc(a,c):
    Fa=np.abs(np.fft.rfft(a-a.mean()))[1:]; Fc=np.abs(np.fft.rfft(c-c.mean()))[1:]; return np.corrcoef(Fa,Fc)[0,1]
def score(Na,Da,label):
    out=[]; tot=0; ex=0
    for cs in runs:
        x,b,Rm,u,tr,trl=cache[cs]
        R=st(L(horner(Na[::-1],x))/L(horner(Da[::-1],x)))
        Bs=st(L(x-1.0)*L(st(L(x-2.0)*L(R))))
        ee=((Rm-tr)-trl)/u; es=((R-tr)-trl)/u
        out.append('%.2f'%fc(ee,es)); ex+=(Bs==b).sum(); tot+=len(b)
    print('%-28s fft %s  Bexact %.3f'%(label,' '.join(out),ex/tot))
if __name__=='__main__':
    for spec in sys.argv[2:]:
        n,d,lo,hi=spec.split(','); n=int(n); d=int(d)
        a,bb,e=fit(n,d,mp.mpf(lo),mp.mpf(hi))
        for sc in (1,):
            score([float(c) for c in a],[float(c) for c in bb],'(%d,%d) on [%s,%s] e%.0e'%(n,d,lo,hi,float(e)))
