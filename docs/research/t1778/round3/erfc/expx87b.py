import numpy as np, mpmath as mp
from multiprocessing import Pool
mp.mp.prec=200
L2E=1/mp.log(2)
with mp.workprec(64): L2E64=+L2E
def f64(v):
    with mp.workprec(64): return +v
def models(zz):
    Z=mp.mpf(zz)
    t=f64(-Z*L2E64)
    A=float(f64(mp.power(2,t)))                 # fl53(fl64(2^t))
    n=mp.nint(t); f=t-n
    u=f64(mp.power(2,f)-1); w=f64(1+u)
    B=float(w*mp.power(2,n))                    # F2XM1 + 1 + FSCALE
    n2=mp.floor(t); f2=t-n2                     # truncating split (f in [0,1))
    u2=f64(mp.power(2,f2)-1); w2=f64(1+u2); C=float(w2*mp.power(2,n2))
    return A,B,C
if __name__=='__main__':
    a=np.concatenate([np.load('runs.npy'),np.load('stepmap.npy')])
    z,E=a[:,3],a[:,2]
    with Pool(4) as p: res=np.array(p.map(models,z,chunksize=2000))
    for j,nm in enumerate(['fl64(2^t)','F2XM1 nint split','F2XM1 floor split']):
        ok=res[:,j]==E; print(nm,'match',ok.sum(),'/',len(E))
        bad=np.where(~ok)[0][:8]
        for i in bad: print('    z=',z[i].hex(),'excel',E[i].hex(),'model',res[i,j].hex())
    np.save('exp_models.npy',res)
