# For each split: interval of relative error rho (units 2^-64) of the value 1+u fed to RN64, consistent with
# Excel E = RN53(RN64(2^f (1+rho)) * 2^n). Reports the minimum |rho| each point requires; scans rho on a fine grid.
import mpmath as mp, numpy as np, sys
from multiprocessing import Pool
mp.mp.prec=220
L2E=1/mp.log(2)
with mp.workprec(64): L2E64=+L2E
def r64(v):
    with mp.workprec(64): return +v
SPL={'floor':mp.floor,'nint':mp.nint,'ceil':mp.ceil}
RG=np.arange(-3,3.0001,0.0625)
def work(line):
    ah,eh,tag=line.split(); a=float.fromhex(ah); E=float.fromhex(eh)
    if E<2.0**-1022: return None
    t=r64(mp.mpf(a)*L2E64); out=[a]
    # quick screen: distance of 2^t to midpoint
    v=mp.power(2,t); m,e=mp.frexp(v); U=mp.mpf(2)**(e-53); d=float((v/U)%1-mp.mpf(0.5))
    if abs(d)>0.004: return None
    for s in ['floor','nint','ceil']:
        n=SPL[s](t); f=t-n; w=mp.power(2,f); sc=mp.mpf(2)**int(n)
        ok=[float(r64(w*(1+mp.mpf(r)*mp.mpf(2)**-64))*sc)==E for r in RG]
        out.append(ok)
    return (a,float(t-mp.floor(t)),out[1:],tag)
if __name__=='__main__':
    L=open('expdata.txt').read().split('\n')[:-1]
    with Pool(4) as p: R=[r for r in p.map(work,L,chunksize=2000) if r is not None]
    import pickle; pickle.dump((RG,R),open('rho.pkl','wb'))
    print('screened',len(R))
    for j,s in enumerate(['floor','nint','ceil']):
        need=[]
        for r in R:
            ok=np.array(r[2][j])
            need.append(np.min(np.abs(RG[ok])) if ok.any() else 99)
        need=np.array(need); print(s,'points needing |rho|>0.25:',(need>0.25).sum(),' >0.5:',(need>0.5).sum(),' >1:',(need>1).sum(),'>2:',(need>2).sum(),' max',need.max())
