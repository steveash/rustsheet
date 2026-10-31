"""RATE secant variants (start offset, residual stop) on the t3478 RATE sweep vs Excel: the fit lab's
T-2005 model (fv_residual, Excel operators) with T-3348's flat-secant hold and run-on below -1.

Run: cd tools && uv run python -I ../docs/research/t3478/rate_variants.py <repo>
"""
import json, math, collections
from rustsheet_tools.fitlab.kernels.solvers import fv_residual, X87, TWICE, PLAIN, RATE_MAX_STEPS, RATE_CAP_TOLERANCE
from rustsheet_tools.fitlab.sweep import split_args
from rustsheet_tools.snapshot import show_cells
import sys

REPO = sys.argv[1]  # the repository root
P=REPO+'/eval/suites/probes-numeric/cases/t3478-rate-r4-wide'
case=json.load(open(P+'.case.json')); cells=case['sheets'][0]['cells']
orc=json.load(open(P+'.oracle.json'))['sheets']['Calc']
eng=show_cells('probes-numeric/t3478-rate-r4-wide.case.json')
def model(res_fn, args, off, res):
    a=tuple(args[:5]); g=args[5]
    r0=g; f0=res_fn(r0,a); r1=r0-off*math.copysign(1.0,f0); f1=res_fn(r1,a)
    for step in range(1,RATE_MAX_STEPS+1):
        if f1==f0:  # T-3348 hold at r1
            ok=(abs(f1)<RATE_CAP_TOLERANCE and 1+r1>RATE_CAP_TOLERANCE) and f1!=0
            return r1 if ok else None
        r2=X87.add(r1,-X87.div(X87.mul(f1,X87.add(r1,-r0)),X87.add(f1,-f0)))
        if not math.isfinite(r2): return None
        f2=res_fn(r2,a)
        size=abs(r2-r1)
        if size<1e-7 and abs(f2)<res: return r2 if r2>-1 else None
        if step==RATE_MAX_STEPS and size<RATE_CAP_TOLERANCE and abs(f2)<RATE_CAP_TOLERANCE and 1+r2>RATE_CAP_TOLERANCE: return r2
        r0,f0,r1,f1=r1,f1,r2,f2
    return None
best=None
for ops in (PLAIN,TWICE):
  for order in ("pv,pmt,fv","pv,fv,pmt","pmt,fv,pv","fv,pv,pmt"):
    rf=fv_residual(ops,order)
    agree=0
    for a,f in cells.items():
        args=[float(x) for x in split_args(f[6:-1])]
        try: r=model(rf,args,1e-3,1e-7)
        except Exception: r=None
        ea=eng['Calc!'+a]['actual']
        try: e=float(ea)
        except ValueError: e=None
        agree+= (r==e)
    print('model', 'twice' if ops is TWICE else 'plain', order, 'agrees with engine', agree)
    if best is None or agree>best[0]: best=(agree,rf)
rf=best[1]
for name,(off,res) in {'shipped +-1e-3 toward smaller |f|, 1e-7':(1e-3,1e-7),'offset 1e-4':(1e-4,1e-7),'offset -1e-3 (other side)':(-1e-3,1e-7),'residual 1e-6':(1e-3,1e-6),'residual 1e-8':(1e-3,1e-8)}.items():
    c=collections.Counter()
    for a,f in cells.items():
        args=[float(x) for x in split_args(f[6:-1])]
        try: r=model(rf,args,off,res)
        except Exception: r=None
        o=orc[a]; ex=o['v'] if o['t']=='n' else None
        ok=(r is None) if ex is None else (r is not None and r==ex)
        c['exact']+=ok
        if ex is not None: c['exact on Excel numbers']+=ok
        c['excel numbers']+= ex is not None
    print(name, dict(c))
