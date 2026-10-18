# For every point: t64 = fl64(a*L2E64), exact v=2^t64, d = position of v relative to nearest midpoint (ulp53 units,
# signed: >0 means v above the midpoint), and Excel's direction (up = Excel is the double above the midpoint).
import mpmath as mp, numpy as np
from multiprocessing import Pool
mp.mp.prec=200
L2E=1/mp.log(2)
with mp.workprec(64): L2E64=+L2E
def r64(v):
    with mp.workprec(64): return +v
def work(line):
    ah,eh,tag=line.split(); a=float.fromhex(ah); E=float.fromhex(eh)
    if E==0: return None
    t=r64(mp.mpf(a)*L2E64); v=mp.power(2,t)
    m,e=mp.frexp(v)   # v=m*2^e, m in [0.5,1)
    if e-53 < -1074: return None
    u=mp.mpf(2)**(e-53)
    k=mp.floor(v/u)   # lower double index
    lo=k*u; mid=lo+u/2
    d=float((v-mid)/u)
    up = 1 if mp.mpf(E)==lo+u else (0 if mp.mpf(E)==lo else -9)
    fl=float(t-mp.floor(t)); 
    ex=float(mp.exp(a)/v-1)   # rel diff true exp vs 2^t64
    return (a,float(t),fl,d,up,E)
if __name__=='__main__':
    L=open('expdata.txt').read().split('\n')[:-1]
    with Pool(8) as p: R=[r for r in p.map(work,L,chunksize=2000) if r is not None]
    R=np.array(R); np.save('cons.npy',R)
    d=R[:,3]; up=R[:,4]
    print('n',len(R),'bad dir (not adjacent)',(up==-9).sum())
    cr=(d>0).astype(int)
    print('exact 2^t64 correctly rounded mismatches',(cr!=up).sum())
    mis=np.where(cr!=up)[0]
    for i in mis: print(f'a={R[i,0]:.6f} f={R[i,2]:.6f} d={d[i]:+.6f} excel_up={int(up[i])}')
