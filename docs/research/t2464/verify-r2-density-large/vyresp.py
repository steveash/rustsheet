"""Own check of the q = fl(1-x) response: per run block, OLS of Excel's relative error (units 2^-53, vs exact pdf at
the double x) on the relative rounding of fl(1-x).  Loader predicts (b-1)-(a+b-2)y; log form b-1; brcomp/(xy) -1."""
import pickle, sys, os, math
from collections import defaultdict
import mpmath
mpmath.mp.prec = 200
U = pickle.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "scored_default.pkl"), "rb"))
bl = defaultdict(list)
for r in U:
    if r["src"].startswith("w2:t2464-w2f-xruns"):
        bl[(r["a"], r["b"], round(r["x"], 3))].append(r)
for key, rs in sorted(bl.items()):
    if len(rs) < 200: continue
    X, Y = [], []
    for r in rs:
        x = mpmath.mpf(r["x"]); a, b = mpmath.mpf(r["a"]), mpmath.mpf(r["b"])
        ex = x ** (a - 1) * (1 - x) ** (b - 1) / mpmath.beta(a, b)
        Y.append(float((mpmath.mpf(r["excel"]) - ex) / ex * 2 ** 53))
        X.append(float((mpmath.mpf(1.0 - r["x"]) - (1 - x)) / (1 - x) * 2 ** 53))
    n = len(X); mx = sum(X) / n; my = sum(Y) / n
    sxx = sum((u - mx) ** 2 for u in X)
    if sxx == 0: continue
    beta = sum((u - mx) * (v - my) for u, v in zip(X, Y)) / sxx
    res = [v - my - beta * (u - mx) for u, v in zip(X, Y)]
    se = math.sqrt(sum(e * e for e in res) / (n - 2) / sxx)
    a, b, y = key[0], key[1], 1 - key[2]
    print(f"({a:g},{b:g}) x~{key[2]} n={n} coef {beta:+.2f} (se {se:.2f})  Loader {(b-1)-(a+b-2)*y:+.2f}  log {b-1:+.2f}  brcomp -1")
