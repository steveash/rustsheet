import json, math, struct
from fractions import Fraction as F
CASES='/home/user/rustsheet/eval/suites/probes-numeric/cases/'
def rnd(q,p):
    """round positive Fraction q to p significant bits, RNE; returns Fraction"""
    if q==0: return q
    s=1
    if q<0: s=-1;q=-q
    e=q.numerator.bit_length()-q.denominator.bit_length()
    if F(2)**e>q: e-=1
    # q in [2^e,2^(e+1))
    sc=p-1-e
    v=q*F(2)**sc
    n,r=divmod(v.numerator,v.denominator)
    r2=2*r
    if r2>v.denominator or (r2==v.denominator and n%2==1): n+=1
    return s*F(n)/F(2)**sc
def fl(q): return rnd(q,53)
def fl6453(q): return rnd(rnd(q,64),53)
def nextup(x): return math.nextafter(x,math.inf)
def nextdn(x): return math.nextafter(x,-math.inf)
def load(name):
    c=json.load(open(CASES+name+'.case.json'))
    o=json.load(open(CASES+name+'.oracle.json'))['sheets']['Calc']
    cells=c['sheets'][0]['cells']
    rows=[]
    i=1
    while ('A%d'%i) in cells:
        x=cells['A%d'%i]; b=o.get('B%d'%i); cc=o.get('C%d'%i)
        if b and cc and b.get('t')=='n' and cc.get('t')=='n':
            rows.append((float(x),b['v'],cc['v']))
        i+=1
    return rows
def recoverR(y,E):
    Fy=F(y); FE=F(E)
    r0=y/E
    cands=[]
    r=r0
    for _ in range(4): r=nextdn(r)
    for _ in range(9):
        if fl6453(FE*F(r))==Fy: cands.append(r)
        r=nextup(r)
    return cands
ALPHA=float.fromhex('0x1.4dc756d76135p-2'); BETA=float.fromhex('0x1.27cdbb941913dp-2')
