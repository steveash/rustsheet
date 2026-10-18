import mpmath as mp, numpy as np, random
exec(open('uerr.py').read().split("G=mp.mpf")[0])
mp.mp.prec=200
H=open('hw.txt').read().split('\n')[:-1]; random.seed(1); S=random.sample(H,30000)
errs=[]; nonrn=0
for lh in S:
    p=lh.split(); f=ldhex(p[3]); u=ldhex(p[4])
    us=mp.power(2,f)-1
    if us==0: continue
    e=float((u-us)/ulp64(r64(us))); errs.append(e); nonrn+= (u!=r64(us))
errs=np.array(errs); print('n',len(errs),'max|err| ulp(u)',abs(errs).max(),'not RN:',nonrn)
