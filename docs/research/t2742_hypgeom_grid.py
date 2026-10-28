"""T-2742: analysis of probes-numeric/t2742-hypgeom-small-n (HYPGEOM.DIST, N 6-12) against exact rationals.

Run from the repository root: python3 docs/research/t2742_hypgeom_grid.py. Prints the bit-exact count of
each candidate formulation, the cdf rules, the n/K symmetry count, the largest ulp offsets and the
offset histogram by N and x position (first, interior, last); the numbers in statistics.md
"HYPGEOM.DIST is not exact for small integer arguments" come from this output.
"""
import json, math, re
from fractions import Fraction as F
from math import comb, lgamma, exp
c=json.load(open('eval/suites/probes-numeric/cases/t2742-hypgeom-small-n.case.json'))['sheets'][0]['cells']
o=json.load(open('eval/suites/probes-numeric/cases/t2742-hypgeom-small-n.oracle.json'))['sheets']['Sheet1']
def ulps(a,b):
    if a==b: return 0
    return round((a-b)/math.ulp(b))
rows=[]
r=1
while f"A{r}" in c:
    x,n,K,N=map(int,re.findall(r'\d+',c[f"A{r}"])[:4])
    rows.append((r,x,n,K,N)); r+=1
def val(a): return o[a]['v'] if o[a]['t']=='n' else None
stats={}
cands={
 'exact':lambda x,n,K,N: float(F(comb(K,x)*comb(N-K,n-x),comb(N,n))),
 'combratio':lambda x,n,K,N: float(comb(K,x))*float(comb(N-K,n-x))/float(comb(N,n)),
 'combratio2':lambda x,n,K,N: float(comb(K,x))*(float(comb(N-K,n-x))/float(comb(N,n))),
 'combratio3':lambda x,n,K,N: (float(comb(K,x))/float(comb(N,n)))*float(comb(N-K,n-x)),
 'lgamma':lambda x,n,K,N: exp(lgamma(K+1)-lgamma(x+1)-lgamma(K-x+1)+lgamma(N-K+1)-lgamma(n-x+1)-lgamma(N-K-n+x+1)-lgamma(N+1)+lgamma(n+1)+lgamma(N-n+1)),
}
cnt={k:0 for k in cands}; tot=0; offs={}
cdf_ok=0; cdf_tot=0; leg_ok=0; leg_tot=0
prev=None; run=0.0
for (r,x,n,K,N) in rows:
    a=val(f"A{r}"); b=val(f"B{r}")
    if a is None: continue
    tot+=1
    for k,f in cands.items():
        if f(x,n,K,N)==a: cnt[k]+=1
    ex=cands['exact'](x,n,K,N); u=ulps(a,ex)
    lo=max(0,n+K-N); hi=min(n,K)
    pos='first' if x==lo else ('last' if x==hi else 'interior')
    offs.setdefault((N,pos),[]).append(u)
    key=(n,K,N)
    if key!=prev: run=0.0; prev=key
    run+=a
    cdf_tot+=1
    if b==run: cdf_ok+=1
    if f"C{r}" in c:
        leg_tot+=1; leg_ok+= (val(f"C{r}")==a)
print('pmf cells',tot, cnt)
print('cdf==running sum of excel pmf', cdf_ok,'/',cdf_tot, ' cdf==1 at last?')
print('legacy==pmf', leg_ok,'/',leg_tot)
from collections import Counter
for N in range(6,13):
    print(N, {p: dict(Counter(offs.get((N,p),[])).most_common(5)) for p in ('first','interior','last')})
pm={}; cd={}
for (r,x,n,K,N) in rows:
    pm[(x,n,K,N)]=(r,val(f"A{r}")); cd[(x,n,K,N)]=(r,val(f"B{r}"))
sym=sum(1 for (x,n,K,N),(r,v) in pm.items() if pm[(x,K,n,N)][1]==v)
print('symmetric n<->K', sym,'/',len(pm))
allu=[(abs(ulps(v,cands['exact'](*k))),k,r,v) for k,(r,v) in pm.items()]
allu.sort(reverse=True); print('max', allu[:3])
cex=sum(1 for (x,n,K,N),(r,v) in cd.items() if v==float(sum(F(comb(K,i)*comb(N-K,n-i),comb(N,n)) for i in range(max(0,n+K-N),x+1))))
print('cdf exact', cex)
lastone=sum(1 for (x,n,K,N),(r,v) in cd.items() if x==min(n,K) and v==1.0); lasts=sum(1 for (x,n,K,N) in cd if x==min(n,K))
print('cdf last ==1', lastone,'/',lasts)
print('cite', pm[(0,3,1,6)], pm[(2,2,3,11)] if (2,2,3,11) in pm else None, cands['exact'](2,2,3,11))
