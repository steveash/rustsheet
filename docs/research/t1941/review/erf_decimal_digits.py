# T-1941 review (Fable 5.1): shortest round-trip decimal length of the bit-exact T-1778 coefficients (a "short decimal literal" prior?).
import re,glob
from decimal import Decimal
def sig(d):
    s=repr(d); m=re.match(r'-?([0-9.]+)(e[-+]?\d+)?',s); digs=m.group(1).replace('.','').lstrip('0').rstrip('0'); return len(digs)
sets={}
for f in ['../../t1778/round3/sol100.txt','../../t1778/round9/erfc_mid_coef.txt','../../t1778/round7/erfc_tail_coef.txt']:
    for l in open(f):
        if l.startswith('#'): continue
        toks=[t for t in l.split() if t.startswith('0x') or t.startswith('-0x')]
        if toks: sets[f.split('/')[-2]+':'+l.split()[0]]=[float.fromhex(t) for t in toks]
import random
random.seed(0); gen=[random.uniform(0.3,3)*10**random.randint(-5,5) for _ in range(20000)]
from collections import Counter
print('generic doubles: digits histogram',sorted(Counter(sig(v) for v in gen).items()))
for k,v in sets.items():
    print(k,[sig(c) for c in v])
# Excel constants from the README: ln sqrt(2pi) literal, sqrt(2pi), sqrt(pi) double, Euler gamma double
for nm,h in [('sqrt(pi)','0x1.c5bf891b4ef6bp+0')]:
    print(nm,sig(float.fromhex(h)),repr(float.fromhex(h)))
print('0.9189385332046727 ->',sig(0.9189385332046727),'; 2.50662827463100050242 ->',sig(2.50662827463100050242),repr(2.50662827463100050242))
