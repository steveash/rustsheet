import math, sys
sys.path.insert(0, '.')
import w5lib as L
import mpmath
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a = c["args"]
    x, d1, d2 = a[0], float(math.trunc(a[1])), float(math.trunc(a[2]))
    kind = "cdf" if c["fn"] == "F.DIST" else "rt"
    if c["fn"] == "F.DIST" and a[3] is not True: continue
    if not (x > 0) or (d1 != 2 and d2 != 2): continue
    p = d1 * x; s = p + d2; w = p / s; v = d2 / s
    if d1 == 2:
        P = L.EXP(d2 / 2 * L.LN(v)); m = 1 - P if kind == "cdf" else P
        ex = mpmath.mpf(v) ** (d2 / 2); exv = 1 - ex if kind == "cdf" else ex
    else:
        P = L.EXP(d1 / 2 * L.LN(w)); m = P if kind == "cdf" else 1 - P
        ex = mpmath.mpf(w) ** (d1 / 2); exv = ex if kind == "cdf" else 1 - ex
    d = L.ulpd(m, c["excel"])
    tag = sys.argv[1] if len(sys.argv) > 1 else "a1"
    if (tag == "a1") != (d1 == 2): continue
    if d:
        print(f"{kind} x={x!r} d1={d1:g} d2={d2:g} w={w:.6g} excel={c['excel']!r} model_ulp={(m-c['excel'])/math.ulp(c['excel']):+.1f} exact(v)_ulp={(float(exv)-c['excel'])/math.ulp(c['excel']):+.1f}  {c['case'].split('/')[-1][:24]} {c['cell']}")
