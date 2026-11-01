"""T-2507: score a correctly rounded BETA.DIST pdf (mpmath, 40 digits, exact 1 - x) against Excel on probes-numeric cases.

Usage: python cr_score.py <case-name>... (run from the repo root; needs mpmath). Prints cells, exact, 4 ulp and
15 shown digits per case. A separate run from ../t2464/exact_ref.py (60 digits, beta_exact.tsv).
"""
import json,sys,math,mpmath,re
from fractions import Fraction
mpmath.mp.dps=40
def vis(e,x): return float('%.15g'%e)!=float('%.15g'%x)
def ev(s):
    return float(eval(s.replace('^','**'),{}))
for name in sys.argv[1:]:
    c=json.load(open(f'eval/suites/probes-numeric/cases/{name}.case.json'))['sheets'][0]
    o=json.load(open(f'eval/suites/probes-numeric/cases/{name}.oracle.json'))['sheets'][c['name']]
    n=ex=v=u4=0
    for k,f in c['cells'].items():
        m=re.fullmatch(r'=BETA\.DIST\((.*),(FALSE|0)\)',f) if isinstance(f,str) else None
        if not m or k not in o or o[k].get('t')!='n': continue
        parts=[];d=0;cur=''
        for ch in m.group(1):
            if ch==',' and d==0: parts.append(cur);cur='';continue
            d+=ch=='(';d-=ch==')';cur+=ch
        parts.append(cur)
        if len(parts)!=3: continue
        try: x,a,b=map(ev,parts)
        except Exception: continue
        if not(a>1 and b>1 and 0<x<1): continue
        X=mpmath.mpf(x)
        ref=float(X**(a-1)*(1-X)**(b-1)/mpmath.beta(a,b))
        e=o[k]['v']; n+=1; ex+=e==ref; v+=not vis(e,ref); u4+=abs(e-ref)<=4*math.ulp(ref)
    print(name,n,'exact',ex,'4ulp',u4,'15dig',v)
