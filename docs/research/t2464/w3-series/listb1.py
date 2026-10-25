"""be b<1 (a>1) direct cells: E - model ulps under z53 and z64 (both with big8), and CR."""
from lib import *
from model import model
rows = [r for r in load(cum=True, branch="direct") if r["b"] < 1 < r["a"] and r["src"] == "be"]
seen = set(); rr = []
for r in rows:
    k = (r["x"], r["a"], r["b"])
    if k not in seen: seen.add(k); rr.append(r)
rr.sort(key=lambda r: (r["b"], r["a"], r["x"]))
x = np.array([r["x"] for r in rr]); a = np.array([r["a"] for r in rr]); b = np.array([r["b"] for r in rr])
E = np.array([r["excel"] for r in rr])
m53 = model(x, a, b, big=8.0); m64 = model(x, a, b, big=8.0, zmode="z64")
for i, r in enumerate(rr):
    cr = float(exact_cdf(r["x"], r["a"], r["b"]))
    u = math.ulp(E[i])
    print(f"  b={r['b']:<7g} a={r['a']:<6g} x={r['x']:<8.4g} z53 {(E[i]-m53[i])/u:+6.0f} z64 {(E[i]-m64[i])/u:+6.0f}  CR {(E[i]-cr)/u:+5.0f}")
