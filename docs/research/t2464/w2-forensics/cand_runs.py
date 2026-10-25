"""Score published incomplete-beta candidates on the w2f x-run blocks (cdf): bit-exact count and the tie response c
(see tie_response.py) of the candidate itself vs Excel's. A candidate with the right structure must reproduce c."""
import csv, collections, sys, math
import numpy as np, mpmath
sys.path.insert(0, '/home/user/rustsheet/tools/src')
from rustsheet_tools.fitlab.kernels import beta as kb
from common import *
from toms import bratio
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(list)
for r in rows:
    if "xruns" not in r["case"] or r["cum"] != "1": continue
    i = int(r["cell"].split("!A")[1]) - 1
    g[(r["case"][-1], i // 2048)].append(r)
cands = {n: f for n, _, f in kb.ibeta_candidates() if n in (
    "Cephes incbet, x^a by exp(a ln x), Gamma correctly rounded", "closed forms, Cephes fractions, mean switch (best)",
    "LibreOffice GetBetaDist + Excel closed forms", "NR betai eps=3e-16, ln(1-x) by log")}
M = mpmath.mpf
def tie_c(xs, errs):
    d = np.array([float((M(1 - x) - (1 - M(x))) / (1 - M(x))) / U for x in xs]); e = np.array(errs)
    if d.std() == 0: return float("nan")
    up, dn = e[d > 1e-9], e[d < -1e-9]
    return (up.mean() - dn.mean()) / (2 * np.abs(d[np.abs(d) > 1e-9]).mean())
for k in sorted(g):
    rs = g[k]; a, b = float(rs[0]["a"]), float(rs[0]["b"])
    xs = [float(r["x"]) for r in rs]; ex = [float(r["excel"]) for r in rs]
    exm = [M(r["exact"]) for r in rs]
    def rel(vals): return [float((M(v) - t) / t) / U for v, t in zip(vals, exm)]
    out = [f"{k} a={a} b={b} x0={xs[0]:.4f} excel c={tie_c(xs, [float(r['rel']) for r in rs]):+.2f}"]
    models = {"exact": [float(t) for t in exm]}
    models["toms"] = [w for w, _, _ in bratio([(a, b, x, 1 - x) for x in xs])]
    for n, f in cands.items(): models[n[:14]] = [f(x, a, b) for x in xs]
    for n, vals in models.items():
        nex = sum(v == e for v, e in zip(vals, ex))
        w4 = sum(ulpdiff(v, e) <= 4 for v, e in zip(vals, ex))
        out.append(f"   {n:15s} exact {nex:4d} w4 {w4:4d} c={tie_c(xs, rel(vals)):+.2f}")
    print("\n".join(out))
