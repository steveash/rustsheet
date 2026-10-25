"""R-style log branch of the beta density (R dbeta, a <= 2 or b <= 2):
    pdf = exp((a-1)*log(x) + (b-1)*log1p(-x) - lbeta(a,b))
with the pieces swapped between libm/R and Excel's own LN/EXP/GAMMALN/GAMMA. Scored on every pdf block with
min(a,b) <= 2 (w2f + beta_exact)."""
import collections, math, sys
import numpy as np, mpmath
from common import *
from toms import rcall
from pdf_cands import blocks
M = mpmath.mpf
g = blocks()
sel = {k: v for k, v in g.items() if min(v[0][1], v[0][2]) <= 2 or k.startswith("beta_exact")}
pts_all = [p for v in sel.values() for p in v if min(p[1], p[2]) <= 2]
xs = sorted({p[0] for p in pts_all}); ab = sorted({(p[1], p[2]) for p in pts_all})
LNx = dict(zip(xs, xl("LN", xs)))
om = sorted({1 - x for x in xs}); LN1 = dict(zip(om, xl("LN", om)))
vals = sorted({v for a, b in ab for v in (a, b, a + b)})
GL = dict(zip(vals, xl("GAMMALN", vals))); GA = dict(zip(vals, xl("GAMMA", vals)))
rlb = dict(zip(ab, rcall("lbeta", [(a, b, 0, 0) for a, b in ab])))
def lb_xl_gl(a, b): return x87("+", GL[a], x87("-", GL[b], GL[a + b]))
def lb_xl_gl_minfirst(a, b):
    p, q = min(a, b), max(a, b); return x87("+", GL[p], x87("-", GL[q], GL[p + q]))
def lb_exact(a, b): return float(exact_lnbeta(a, b))
lbG_cache = {}
def lb_xl_gamma(a, b):
    p, q = min(a, b), max(a, b)
    return x87("*", GA[p], x87("/", GA[q], GA[p + q]))  # then LN, applied in batch
# exponent assembly
def expo(x, a, b, lnx, l1p, lb, mode):
    if mode == "sse":
        return (a - 1) * lnx + (b - 1) * l1p - lb
    t1 = x87("*", a - 1, lnx); t2 = x87("*", b - 1, l1p)
    return x87("-", x87("+", t1, t2), lb)
variants = {}
lbv = {"R lbeta": lambda a, b: rlb[(a, b)], "xl GL a+(b-s)": lb_xl_gl, "xl GL min+(max-s)": lb_xl_gl_minfirst,
       "exact lnB": lb_exact}
# LN of the Excel GAMMA ratio
ratio = {(a, b): lb_xl_gamma(a, b) for a, b in ab}
lnr = dict(zip(ab, xl("LN", [ratio[k] for k in ab])))
lbv["xl LN(GAMMA ratio)"] = lambda a, b: lnr[(a, b)]
results = collections.defaultdict(lambda: collections.defaultdict(lambda: np.zeros(4, int)))
args_all = []
specs = []
for lname, lf in lbv.items():
    for lnmode in ("libm", "xlLN"):
        for l1mode in ("log1p", "xlLN(1-x)"):
            for mode in ("sse", "x87"):
                specs.append((lname, lf, lnmode, l1mode, mode))
# compute exponents, then one EXP batch per spec
for spec in specs:
    lname, lf, lnmode, l1mode, mode = spec
    E = []
    for k, pts in sel.items():
        for x, a, b, e, t in pts:
            if min(a, b) > 2: continue
            lnx = LNx[x] if lnmode == "xlLN" else math.log(x)
            l1p = LN1[1 - x] if l1mode != "log1p" else math.log1p(-x)
            E.append(expo(x, a, b, lnx, l1p, lf(a, b), mode))
    for expname in ("libm exp", "xl EXP"):
        V = xl("EXP", E) if expname == "xl EXP" else [math.exp(v) for v in E]
        i = 0
        for k, pts in sel.items():
            for x, a, b, e, t in pts:
                if min(a, b) > 2: continue
                v = V[i]; i += 1
                results[spec[0::1][0:1] + spec[2:] + (expname,)][k] += np.array([1, v == e, ulpdiff(v, e) <= 4, within15(v, e)])
best = sorted(results.items(), key=lambda kv: -sum(s[1] for s in kv[1].values()))
for name, per in best[:12]:
    tot = sum(per.values())
    print(name, "TOTAL", tuple(tot))
name, per = best[0]
for k, s in sorted(per.items()): print("   ", k, tuple(s))

print()
keys = sorted(sel)
hdr = [k.split(":")[0][:7] + ":" + k.split(":")[1][-10:] for k in keys]
print("variant".ljust(70), " ".join(h[-12:].rjust(12) for h in hdr))
for name, per in best:
    print(str(name)[:70].ljust(70), " ".join(f"{per[k][1]:>5d}/{per[k][0]:<6d}" for k in keys))
