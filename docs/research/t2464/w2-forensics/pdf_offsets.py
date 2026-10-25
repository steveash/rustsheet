"""Per pdf block: Excel's median relative error vs the -(lnB_model - lnB) offset of candidate Beta functions:
GAMMALN form GL(a)+(GL(b)-GL(a+b)) with Excel's GAMMALN, the two other groupings, TOMS betaln, Excel GAMMA ratio."""
import csv, collections
import numpy as np, mpmath
from common import *
from toms import call
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(list)
for r in rows:
    if r["case"] == "t2464-w2f-unit-shape" or r["cum"] != "0": continue
    i = int(r["cell"].split("!A")[1]) - 1
    size = 2048 if "xruns" in r["case"] else 1024
    g[(r["case"][10:], i // size)].append(r)
M = mpmath.mpf
for k in sorted(g):
    rs = g[k]
    if k[0] == "absweep": continue
    a, b = float(rs[0]["a"]), float(rs[0]["b"])
    e = np.array([float(r["rel"]) for r in rs])
    GLa, GLb, GLs = xl("GAMMALN", [a, b, a + b]); Ga, Gb, Gs = xl("GAMMA", [a, b, a + b])
    lt = exact_lnbeta(a, b)
    def off(v): return -float(M(v) - lt) / U
    c1 = x87("+", GLa, x87("-", GLb, GLs)); c2 = x87("+", GLb, x87("-", GLa, GLs)); c3 = x87("-", x87("+", GLa, GLb), GLs)
    tb = call("betaln", [(a, b, 0, 0)])[0]
    gr = x87("/", x87("*", Ga, Gb), Gs)
    off_gr = -float((M(gr) - mpmath.beta(a, b)) / mpmath.beta(a, b)) / U
    print(f"{k[0]:8s} {k[1]} a={a} b={b} excel median {np.median(e):+6.2f} mean {e.mean():+6.2f} | GL a+(b-s) {off(c1):+7.2f}  b+(a-s) {off(c2):+7.2f}  (a+b)-s {off(c3):+7.2f}  toms {off(tb):+7.2f}  GAMMA {off_gr:+7.2f}")
