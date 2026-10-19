# T-1941 W6: per run, correlation between Excel's rounding residual (B - lnGamma)/ulp and each candidate's simulated
# residual (sim - lnGamma)/ulp, and the |FFT| correlation. A kernel with the right structure correlates strongly even
# when its stage values are a few ulps off; a wrong structure does not.
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import *
mp.mp.dps=40
X,B,C=load(sys.argv[1]); cases=sys.argv[2:]
N,D=read_k3(); P,Q=read_k2()
def run(cs):
    s=(C==cs); x=X[s]; b=B[s]; d=np.diff(x); h=np.median(d)
    br=np.where(d!=h)[0]; seg=np.split(np.arange(len(x)),br+1); seg=max(seg,key=len)
    return x[seg],b[seg],h
for cs in cases:
    x,b,h=run(cs); n=len(x)
    if n<512: continue
    x0=mp.mpf(float(x[0])); H=mp.mpf(float(h)); g=[mp.diff(mp.loggamma,x0,k)/mp.factorial(k) for k in range(4)]
    ex=np.array([float(g[0]+(i*H)*(g[1]+(i*H)*(g[2]+(i*H)*g[3]))-mp.mpf(float(b[i]))) for i in range(n)])  # lnG - B (small)
    ub=np.spacing(np.abs(b))
    re=-ex/ub
    out=[]
    for nm,sim in (('K3',k3(x,N,D)[0]),('K2',k2(x,P,Q)[0])):
        rs=((sim-b)-ex)/ub   # (sim - lnG)/ulp, computed as (sim-B)+(B-lnG) to avoid cancellation
        c=np.corrcoef(re,rs)[0,1]
        Fe=np.abs(np.fft.rfft(re-re.mean())); Fs=np.abs(np.fft.rfft(rs-rs.mean()))
        cf=np.corrcoef(Fe[1:],Fs[1:])[0,1]
        out.append('%s corr %+.3f fftcorr %.3f exact %.3f'%(nm,c,cf,(sim==b).mean()))
    print('%-16s x0=%.6f n=%5d  %s'%(cs,float(x0),n,' | '.join(out)))
