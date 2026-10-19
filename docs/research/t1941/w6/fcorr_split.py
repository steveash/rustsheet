# T-1941 W6: |FFT| correlation of Excel's residual vs a candidate's simulated residual on sub-blocks of one run.
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import *
mp.mp.dps=40
X,B,C=load(sys.argv[1]); cs=sys.argv[2]; blk=int(sys.argv[3])
N,D=read_k3(); P,Q=read_k2()
s=(C==cs); x=X[s]; b=B[s]
def resid(x,b,sim):
    x0=mp.mpf(float(x[0])); g=[mp.diff(mp.loggamma,x0,k)/mp.factorial(k) for k in range(4)]
    ex=np.array([float(g[0]+(mp.mpf(float(xi))-x0)*(g[1]+(mp.mpf(float(xi))-x0)*(g[2]+(mp.mpf(float(xi))-x0)*g[3]))-mp.mpf(float(bi))) for xi,bi in zip(x,b)])
    ub=np.spacing(np.abs(b)); re=-ex/ub; rs=((sim-b)-ex)/ub; return re,rs
def fc(a,c):
    Fa=np.abs(np.fft.rfft(a-a.mean()))[1:]; Fc=np.abs(np.fft.rfft(c-c.mean()))[1:]; return np.corrcoef(Fa,Fc)[0,1]
for i in range(0,len(x)-blk+1,blk):
    xx=x[i:i+blk]; bb=b[i:i+blk]
    out=[]
    for nm,sim in (('K3',k3(xx,N,D)[0]),('K2',k2(xx,P,Q)[0])):
        re,rs=resid(xx,bb,sim); out.append('%s fft %.3f ex %.3f'%(nm,fc(re,rs),(sim==bb).mean()))
    print('%s..%s  %s'%(xx[0].hex(),xx[-1].hex(),' | '.join(out)))
