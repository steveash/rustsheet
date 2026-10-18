# Per point: allowed integer offsets k (|k|<=K) of Excel's 64-bit u relative to RN64(2^f-1), under
# split S (floor/nint/ceil): w=RN64(1+u), E=RN53(w*2^n). Points where not all k allowed are informative.
import mpmath as mp, numpy as np, sys
from multiprocessing import Pool
mp.mp.prec=220
L2E=1/mp.log(2)
with mp.workprec(64): L2E64=+L2E
K=4
SPLIT=sys.argv[1] if len(sys.argv)>1 else 'floor'
def r64(v):
    with mp.workprec(64): return +v
def ulp64(v):
    m,e=mp.frexp(v); return mp.mpf(2)**(e-64)
def work(line):
    ah,eh,tag=line.split(); a=float.fromhex(ah); E=float.fromhex(eh)
    if E==0 or E<2.0**-1022: return None
    t=r64(mp.mpf(a)*L2E64)
    n={'floor':mp.floor,'nint':mp.nint,'ceil':mp.ceil}[SPLIT](t)
    f=r64(t-n) if SPLIT=='floor64' or SPLIT=='floor' else t-n; us=mp.power(2,f)-1
    if us==0: return None
    u0=r64(us); ul=ulp64(u0) if True else None
    sc=mp.mpf(2)**int(n); res=[]
    for k in range(-K,K+1):
        u=u0+k*ul   # (crossing binade ignored; ok for k small unless u0 near power of 2)
        w=r64(1+u); res.append(float(w*sc)==E)
    eu=float((u0-us)/ul)  # RN error of u0 itself
    return (a,float(f),eu,tag,res)
if __name__=='__main__':
    L=open('expdata.txt').read().split('\n')[:-1]
    with Pool(4) as p: R=[r for r in p.map(work,L,chunksize=2000) if r is not None]
    import pickle; pickle.dump(R,open(f'kset_{SPLIT}.pkl','wb'))
    ok0=sum(r[4][K] for r in R); print(SPLIT,'n',len(R),'k=0 (correctly rounded u) consistent:',ok0)
    for r in sorted(R,key=lambda r:r[1]):
        if not all(r[4]):
            s=''.join('o' if x else '.' for x in r[4])
            if not r[4][K] or '-v' in sys.argv: print(f'f={r[1]:+.5f} a={r[0]:10.4f} {s} {r[3]}')
