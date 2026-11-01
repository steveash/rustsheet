"""T-3643 Gate A envelope: relative error against 50-digit math of each engine miss on the fresh sweep.

Run: ./target/release/rustsheet-eval show probes-numeric/t3643-r4-log-modulus.case.json > /tmp/f.txt; then from tools/:
uv run python ../docs/research/t3643/env.py /tmp/f.txt
"""
import re,sys
from mpmath import mp, mpf, log, atan2, sqrt
mp.dps=50
txt=open(sys.argv[1]).read()
def parts(t):
    if t[-1]!='i': return float(t), 0.0
    b=t[:-1]
    for i in range(len(b)-1,0,-1):
        if b[i] in "+-" and b[i-1] not in "Ee": return float(b[:i]), float(b[i:] if b[i:] not in "+-" else b[i:]+"1")
    return 0.0, float(b)
for m in re.finditer(r'(S!\S+)\s+\S+\n\s+formula : =(\w+)\(COMPLEX\(([^,]+),([^)]+)\)\)\n\s+expected: "([^"]+)"\n\s+actual  : "([^"]+)"', txt):
    cell,fn,x,y,ex,ac=m.groups()
    x=mpf(f"{float(x):.15g}"); y=mpf(f"{float(y):.15g}")
    L=log(sqrt(x*x+y*y)); A=atan2(y,x)
    base={"IMLN":1,"IMLOG10":log(10),"IMLOG2":log(2)}[fn]
    mr, mi = L/base, A/base
    ar, ai = parts(ac); er, ei = parts(ex)
    rel=lambda a,m: abs(mpf(a)-m)/max(abs(m),mpf('1e-300'))
    print(cell, fn, "re" if ar!=er else "im", f"engine rel err vs math: re {float(rel(ar,mr)):.2e} im {float(rel(ai,mi)):.2e}", "| excel", ex, "| ours", ac)
