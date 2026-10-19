# T-1941: model-free bias of Excel's GAMMA on [1,2): mean (Excel - Gamma)/Gamma per 0.05 bin. Input: g_rows.py output.
# model-free: mean relative error of Excel's GAMMA(y) vs true Gamma on [1,2), per 0.05 bin (subsample)
import mpmath as mp, random
mp.mp.dps=30
rows=[l.split() for l in open('g_rows.txt')]
random.seed(1); rows=random.sample(rows,60000)
bins={}
for xh,gh,_ in rows:
    y=float.fromhex(xh); g=float.fromhex(gh)
    t=mp.gamma(mp.mpf(y)); e=float((mp.mpf(g)-t)/t)
    bins.setdefault(int((y-1)*20),[]).append(e)
for k in sorted(bins):
    v=bins[k]; m=sum(v)/len(v)
    print('%.2f-%.2f n=%5d mean rel err %+.2e  (%+.2f ulp-rel)'%(1+k/20,1+(k+1)/20,len(v),m,m/2**-53))
