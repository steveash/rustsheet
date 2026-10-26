"""List beta_exact direct cdf cells with min(a,b)<1: x, a, b, E - v1 (ulps), E - CR (ulps), terms."""
import sys
from lib import *
from model import model
from series import accum2
rows = [r for r in load(cum=True, branch="direct") if min(r["a"], r["b"]) < 1 and r["a"] != 1 and r["b"] != 1 and r["src"] == "be"]
seen = set(); rr = []
for r in rows:
    k = (r["x"], r["a"], r["b"])
    if k in seen: continue
    seen.add(k); rr.append(r)
rr.sort(key=lambda r: (r["a"], r["b"], r["x"]))
x = np.array([r["x"] for r in rr]); a = np.array([r["a"] for r in rr]); b = np.array([r["b"] for r in rr])
E = np.array([r["excel"] for r in rr])
m = model(x, a, b)
_, nt = accum2(x, a, b, mode="x87", rec="R4", tf="T0", start="1/a")
for i, r in enumerate(rr):
    cr = float(exact_cdf(r["x"], r["a"], r["b"]))
    print(f"  a={r['a']:<7g} b={r['b']:<7g} x={r['x']:<11.4g} E-v1 {(E[i]-m[i])/math.ulp(E[i]):+8.0f}  E-CR {(E[i]-cr)/math.ulp(E[i]):+5.0f}  terms {nt[i]}")
