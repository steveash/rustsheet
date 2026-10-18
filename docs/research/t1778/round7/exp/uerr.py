# For near-midpoint points: exact u*=2^f-1 (floor split, f in [0,1)), hardware u error (ulp64 of u), and the
# interval of u errors (ulp64 of u) consistent with Excel under w=fl64(1+u), E=fl53(w*2^n).
import mpmath as mp, numpy as np, sys
mp.mp.prec=200
def r64(v):
    with mp.workprec(64): return +v
def ulp64(v):
    m,e=mp.frexp(v); return mp.mpf(2)**(e-64)
def ldhex(s): return mp.mpf(float.fromhex(s)) if False else mp.mpf(int(s.split('p')[0].replace('-0x','').replace('0x','').replace('.',''),16))*(-1 if s.startswith('-') else 1)*mp.mpf(2)**(int(s.split('p')[1])-4*len(s.split('p')[0].split('.')[1]) if '.' in s else int(s.split('p')[1]))
G=mp.mpf(2)**-63
def work(args):
    le,lh=args
    ah,eh,tag=le.split(); a=float.fromhex(ah); E=float.fromhex(eh)
    if E==0: return None
    p=lh.split(); f=ldhex(p[3]); u=ldhex(p[4]); n=int(p[7])
    v=mp.power(2,f)              # exact 2^f in [1,2)
    sc=mp.mpf(2)**n
    m,e=mp.frexp(mp.mpf(E)); U=mp.mpf(2)**(e-53)
    if e-53<-1074: return None
    lo=(mp.mpf(E)-U/2)/sc; hi=(mp.mpf(E)+U/2)/sc   # range of w (tie handling ignored)
    # w lies on grid G for w in [1,2); allowed grid points k with lo<=kG<=hi
    klo=mp.ceil(lo/G); khi=mp.floor(hi/G)
    def ok(k): return float(k*G*sc)==E   # python float() rounds mpf to nearest-even double
    while not ok(klo): klo+=1
    while ok(klo-1): klo-=1
    while not ok(khi): khi-=1
    while ok(khi+1): khi+=1
    # u range: w=fl64(1+u) in [klo,khi]*G  => 1+u in [(klo-.5)G,(khi+.5)G]
    elo=float(((klo-mp.mpf(0.5))*G - v)/G); ehi=float(((khi+mp.mpf(0.5))*G - v)/G)
    ehw=float((1+u-v)/G)
    d=float((v*sc - (mp.mpf(E)))/U)   # pos of exact 2^t relative to Excel value, ulp53
    return (a,float(f),elo,ehi,ehw,d,int(n))
if __name__=='__main__':
    from multiprocessing import Pool
    L=open('expdata.txt').read().split('\n')[:-1]; H=open('hw.txt').read().split('\n')[:-1]
    with Pool(4) as pool: R=[r for r in pool.map(work,list(zip(L,H)),chunksize=2000) if r is not None]
    R=np.array(R); np.save('uerr.npy',R)
    elo,ehi,ehw=R[:,2],R[:,3],R[:,4]
    print('n',len(R),'hw outside excel range',((ehw<elo)|(ehw>ehi)).sum(), ' exact (e=0) outside',((0<elo)|(0>ehi)).sum())
    tight=(ehi-elo)<2000
    print('informative points (range width < 2000 w-ulps):',tight.sum())
