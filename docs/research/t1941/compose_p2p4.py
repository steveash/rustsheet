# T-1941 W-D: composition tests on reviewer probes P2 ((1.5,2] glue), P3 ([3,8) glue), P4 (legacy GAMMALN).
# Every candidate op is rn53(rn64(exact)) (x87 stored); Excel LN model = rn53(rn64(rn64(ln2)*log2 x)).
import json, sys, mpmath
sys.path.insert(0,'docs/research/t1941')
from rnd import rn53, rn64
L2=rn64(mpmath.log(2))
def xln(v): return rn53(rn64(L2*mpmath.log(mpmath.mpf(v),2)))
def st(v): return rn53(rn64(v))
C='eval/suites/probes-numeric/cases/'
def load(name, cols):
    c=json.load(open(C+name+'.case.json'))['sheets'][0]['cells']; o=json.load(open(C+name+'.oracle.json'))['sheets']['Calc']
    out=[]; r=1
    while f'A{r}' in c:
        row={'A':float(c[f'A{r}'])}
        for k in cols:
            v=o.get(f'{k}{r}',{}).get('v'); row[k]=float(v) if isinstance(v,(int,float)) else None
        out.append(row); r+=1
    return out
M=mpmath.mpf
print('== P2 (1.5,2]: B=GAMMALN.PRECISE(x) C=LN(x) E=GAMMALN.PRECISE(x+1) F=GAMMALN.PRECISE(x-1) G=LN(x-1) H=GAMMA(x)')
for nm in ('kmyst-gam-p201','kmyst-gam-p202','kmyst-gam-p203','kmyst-gam-p204'):
    R=[r for r in load(nm,'BCEFGH') if None not in r.values()]
    t=dict(E_minus_C=0,F_plus_G=0,lnH=0,BpC_is_double=0,C_is_xln=0,G_is_xln=0)
    for r in R:
        B,Cc,E,F,G,H=(r[k] for k in 'BCEFGH')
        t['E_minus_C']+= B==st(M(E)-M(Cc))
        t['F_plus_G']+= B==st(M(F)+M(G))
        t['lnH']+= B==xln(H)
        s=M(B)+M(Cc); t['BpC_is_double']+= rn53(s)==s
        t['C_is_xln']+= Cc==xln(r['A'])
        t['G_is_xln']+= G==xln(r['A']-1)
    print(nm,'x0=%.4f n=%d '%(R[0]['A'],len(R))+' '.join('%s %.4f'%(k,v/len(R)) for k,v in t.items()))
print('== P3 [3,8) glue: B == fl(E + F)')
for nm,cols in (('kmyst-gam-p301','BEGF'),('kmyst-gam-p302','BEGHIF')):
    R=[r for r in load(nm,cols) if None not in r.values()]
    ok=sum(r['B']==st(M(r['E'])+M(r['F'])) for r in R)
    print(nm,'n=%d B==fl(E+F) %.4f'%(len(R),ok/len(R)))
print('== P4 legacy: C=GAMMALN(x) vs B=GAMMALN.PRECISE(x); D=GAMMA.DIST(1,x,1,FALSE)')
R=[r for r in load('kmyst-gam-p401','BCDE') if None not in r.values()]
print('n=%d C==B %.4f  max|C-B|/ulp %.1f'%(len(R),sum(r['B']==r['C'] for r in R)/len(R),max(abs(r['C']-r['B'])/__import__('math').ulp(r['B']) for r in R)))
