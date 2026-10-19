# T-1941 W6: per-run complex phase of identified lines in data vs a candidate chain's simulation. For each long run
# and each law (chain, law, j), evaluate the Hann-windowed DFT of both R residuals at the candidate's predicted
# frequency and report the phase difference (cycles) and amplitude ratio. On a synthetic control, compare with the
# phase difference predicted from (truth - candidate) stage values: for a sum line, d(S/u); triangle d(S/(2u));
# sideband P_j n: d(P/u_P) - n*d(S_{j+1}/u_M).
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import st, L, horner; from synth import read_chain; from longspec import X0, extractR, resid
from stagelaws import stage_F
def sim_resid(N,D,x):
    R=st(L(horner(list(N[::-1]),x))/L(horner(list(D[::-1]),x))); B=st(L(x-1.0)*L(st(L(x-2.0)*L(R)))); lo,hi=extractR(x,B); return resid(x,lo,hi)
def dft_at(e,f):
    n=len(e); w=np.hanning(n); k=np.arange(n); return np.sum((e-e.mean())*w*np.exp(-2j*np.pi*f*k))/w.sum()*2
mp.mp.dps=50
def _stages_mp(c,x0):
    c=[mp.mpf(float(v)) for v in c]; x=mp.mpf(float(x0)); n=len(c)-1; s=c[n]; out={}
    for j in range(n-1,-1,-1):
        P=s*x; out['P%d'%j]=P; s=P+c[j]; out['S%d'%j]=s
    return out
def _ulp(v): return mp.mpf(np.spacing(abs(float(v))))
def stage_phase(c,law,j,x0):
    sv=_stages_mp(c,x0)
    if law in ('S','T'):
        v=sv['S%d'%j]; return float((v/_ulp(v)/(2 if law=='T' else 1))%1)
    nn=int(law[2:]); P=sv['P%d'%j]; M=sv['S%d'%(j+1)]
    return float((P/_ulp(P)-nn*M/_ulp(M))%1)
if __name__=='__main__':
    Nd,Dd=read_chain(sys.argv[1]); Nc,Dc=read_chain(sys.argv[2]); Nd=np.array(Nd)/Dd[0]; Dd=np.array(Dd)/Dd[0]; Nc=np.array(Nc)/Dc[0]; Dc=np.array(Dc)/Dc[0]
    laws=[tuple(s.split(':')) for s in sys.argv[3].split(',')]; nrun=int(sys.argv[4]); n=int(sys.argv[5]) if len(sys.argv)>5 else 65536
    for i in range(nrun):
        x=np.empty(n); x[0]=X0[i]
        for k in range(1,n): x[k]=np.nextafter(x[k-1],3.0)
        ed=sim_resid(Nd,Dd,x); ec=sim_resid(Nc,Dc,x); line=[]
        for ch,law,j in laws:
            j=int(j); c=Nc if ch=='N' else Dc; key=('S%d'%j if law=='S' else 'T%d'%j) if law in ('S','T') else 'P%dn%s'%(j,law[2:])
            f=stage_F(c,np.array([x[0]]))[key][0][0]; fa=f%1.0
            a=dft_at(ed,fa); b=dft_at(ec,fa)
            meas=(np.angle(a/b)/(2*np.pi)); pred=(stage_phase(Nd if ch=="N" else Dd,law,j,x[n//2])-stage_phase(c,law,j,x[n//2]))
            pw=((pred+0.5)%1)-0.5; line.append('%s.%s%d meas %+.3f pred %+.3f (%+.3f) |r| %.2f'%(ch,law,j,meas,pw,((-pred+0.5)%1)-0.5,abs(a)/max(abs(b),1e-9)))
        print('run %d x0 %.4f: '%(i,X0[i])+' | '.join(line),flush=True)
