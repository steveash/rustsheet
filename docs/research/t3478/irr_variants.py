"""Run IRR secant-in-d variants (offset, residual, step cap) on the t3478 IRR sweep vs Excel.

Run: cd tools && uv run python -I ../docs/research/t3478/irr_variants.py <repo>
"""
import json, sys, math, collections
from rustsheet_tools.fitlab.kernels import irr_secant_d as m
from rustsheet_tools.fitlab.kernels.solvers import X87
from rustsheet_tools.snapshot import show_cells
import sys

REPO = sys.argv[1]  # the repository root
P=REPO+'/eval/suites/probes-numeric/cases/t3478-irr-r4-wide'
case=json.load(open(P+'.case.json')); cells=case['sheets'][0]['cells']
orc=json.load(open(P+'.oracle.json'))['sheets']['Calc']
eng=show_cells('probes-numeric/t3478-irr-r4-wide.case.json')
npv=m._npv_v_running; div=X87.div
def run(vals,g,off,res,cap):
    largest=max(abs(c) for c in vals)
    d0=div(g,1+g); f0=npv(vals,1-d0); d1=d0-(off if f0>0 else -off); f1=npv(vals,1-d1)
    for _ in range(cap):
        if f1==f0: return None
        d2=d1-div(f1*(d1-d0),f1-f0)
        if not math.isfinite(d2) or d2>=1: return None
        f2=npv(vals,1-d2)
        st=d2==d1 and abs(f2)<1e-7*largest
        if abs(d2-d1)<1e-7 and (abs(f2)<res or st):
            x=div(1.0,1-d2); return x-1 if x<1e7 else None
        d0,f0,d1,f1=d1,f1,d2,f2
    return None
def irr(vals,g,off,res,cap):
    i=next((i for i,c in enumerate(vals) if c!=0),0); vals=vals[i:]
    try: r=run(vals,g,off,res,cap)
    except (ZeroDivisionError,OverflowError): r=None
    if r is None and g!=0.1:
        try: r=run(vals,0.1,off,res,cap)
        except (ZeroDivisionError,OverflowError): r=None
    return r
variants={'shipped (1e-3, 1e-7, 50)':(1e-3,1e-7,50),'offset 1e-4':(1e-4,1e-7,50),'offset 1e-2':(1e-2,1e-7,50),'residual 1e-6':(1e-3,1e-6,50),'residual 1e-8':(1e-3,1e-8,50),'cap 100':(1e-3,1e-7,100),'cap 1000':(1e-3,1e-7,1000)}
score=collections.defaultdict(collections.Counter)
for a,f in cells.items():
    body=f[5:-1]; arr=body[body.index('{')+1:body.index('}')]
    vals=[float(x) for x in arr.split(',')]; rest=body[body.index('}')+1:]
    g=float(rest[1:]) if rest.startswith(',') else 0.1
    o=orc[a]; ex=o['v'] if o['t']=='n' else None
    st=case['hypotheses']['Calc!'+a].split('/',1)[1]
    ea=eng['Calc!'+a]['actual']
    try: e=float(ea)
    except ValueError: e=None
    for name,(off,res,cap) in variants.items():
        r=irr(vals,g,off,res,cap)
        if r is None and e is None or (r is not None and e is not None and r==e): score[name]['==engine']+=1
        if ex is None: ok = r is None and o.get('v') == '#NUM!'  # a failed run is #NUM!
        else: ok = r is not None and r==ex
        score[name]['exact']+=ok
        score[name][st]+=ok
for k,c in score.items(): print(k, dict(c))
