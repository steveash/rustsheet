"""Does Excel's direct cdf share its pdf's error?  Per run block: sd of rel err of cdf, pdf, and cdf/pdf
(against exact cdf/pdf), plus correlation of cdf and pdf errors.  Units 2^-53."""
import csv, collections
import numpy as np
rows = list(csv.DictReader(open("../w2-forensics/exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(dict)
for r in rows:
    if "xruns" not in r["case"]: continue
    i = int(r["cell"].split("!A")[1]) - 1
    blk = (r["case"][-1], i // 2048)
    g[blk].setdefault(r["x"], {})[r["cum"]] = float(r["rel"])
for blk in sorted(g):
    d = g[blk]; xs = [x for x in d if '0' in d[x] and '1' in d[x]]
    c = np.array([d[x]['1'] for x in xs]); p = np.array([d[x]['0'] for x in xs])
    q = c - p
    print(blk, len(xs), f"sd cdf {c.std():5.2f} pdf {p.std():5.2f} cdf-pdf {q.std():5.2f} corr {np.corrcoef(c,p)[0,1]:+.2f}  means {c.mean():+.2f} {p.mean():+.2f}")
