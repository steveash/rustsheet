import numpy as np, mpmath as mp, sys
mp.mp.prec=200
a=np.concatenate([np.load('runs.npy'),np.load('stepmap.npy')])
x,y,E,z=a[:,0],a[:,1],a[:,2],a[:,3]
def r64(v, mode='n'):
    # round mpf to 64-bit mantissa
    with mp.workprec(64): return mp.mpf(v, rounding=mode) if False else +v
L2E_true=1/mp.log(2)
with mp.workprec(64): L2E64=+L2E_true
LN2=mp.log(2)
with mp.workprec(64): LN2_64=+LN2
print('L2E64 rel err', float((L2E64-L2E_true)/L2E_true))
def model(zz, variant):
    Z=mp.mpf(zz)
    if variant=='cr': return float(mp.exp(-Z))
    if variant=='l2e64':   # t = fl64(-z*L2E64); 2^t exact; round to 53
        with mp.workprec(64): t=-Z*L2E64
        return float(mp.power(2,t))
    if variant=='l2e64_f64':   # also final 2^t rounded to 64 first
        with mp.workprec(64): t=-Z*L2E64
        v=mp.power(2,t)
        with mp.workprec(64): v=+v
        return float(v)
    if variant=='l2e_true_t64':
        with mp.workprec(64): t=-Z*L2E_true
        return float(mp.power(2,t))
    if variant=='l2e64_exact_t':
        return float(mp.power(2,-Z*L2E64))
N=len(z); idx=np.random.default_rng(0).choice(N,20000,replace=False)
for v in ['cr','l2e64','l2e64_f64','l2e_true_t64','l2e64_exact_t']:
    pr=np.array([model(z[i],v) for i in idx])
    ok=(pr==E[idx]); print(f'{v:16s} match {ok.sum()}/{len(idx)}  mismatches {(~ok).sum()}')
