"""README-branch scores (direct: x <= a/(a+b), reflected otherwise) of the bratio-map model vs correctly rounded,
on beta_exact cells only and on all cells (beta_exact + w2 probes), with the hold-out half. Closed forms a=1/b=1
and max(a,b) > 1e4 excluded (as in bratio_map.py)."""
import sys
from lib import *
ZM = sys.argv[1] if len(sys.argv) > 1 else "z53"
rows = [r for r in load(cum=True) if r["a"] != 1.0 and r["b"] != 1.0 and max(r["a"], r["b"]) <= 1e4]
pred = np.load(HERE / f"bratio_pred_{ZM}.npy")
E = np.array([r["excel"] for r in rows])
cr = np.array([float(exact_cdf(r["x"], r["a"], r["b"])) for r in rows])
tune, hold = split_half(rows); tid = {id(r) for r in tune}
for src in ("be", "all"):
    for br in ("direct", "reflected"):
        idx = np.array([i for i, r in enumerate(rows) if r["branch"] == br and (src == "all" or r["src"] == "be")])
        ih = np.array([i for i in idx if id(rows[i]) not in tid])
        print(f"  {src:3s} {br:9s} model {fmt(score(pred[idx], E[idx])):24s} hold {fmt(score(pred[ih], E[ih])):24s} | CR {fmt(score(cr[idx], E[idx])):24s} hold {fmt(score(cr[ih], E[ih]))}")
