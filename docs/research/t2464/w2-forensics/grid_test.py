"""Is Excel's direct-branch cdf a multiple of 2^-53 (the value of 1 - w1 with w1 in [0.5,1))? Count residues of
Excel's value mod 2^-53 / ulp, per block."""
import csv, collections, math
from fractions import Fraction
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(list)
for r in rows:
    if r["case"] == "t2464-w2f-unit-shape": continue
    i = int(r["cell"].split("!A")[1]) - 1
    size = 2048 if "xruns" in r["case"] else 1024
    g[(r["case"][10:], i // size, r["cum"])].append(r)
for k in sorted(g):
    if k[2] != "1": continue
    rs = g[k]
    vals = [float(r["excel"]) for r in rs]; ex = [float(r["exact"]) for r in rs]
    def frac(v, q):  # fraction of values that are multiples of q
        return sum(Fraction(x) % q == 0 for x in v) / len(v)
    v0 = vals[len(vals)//2]
    print(k, f"a={rs[0]['a']} b={rs[0]['b']} I~{v0:.4g} ulp=2^{int(math.log2(math.ulp(v0)))}",
          " ".join(f"m2^-{j}:{frac(vals, Fraction(1, 2**j)):.2f}/{frac(ex, Fraction(1, 2**j)):.2f}" for j in (53, 54, 55, 56)))
