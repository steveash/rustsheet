"""Sanity checks on ix_evidence.tsv: |err_ulp| by confidence and source; cross-check T.DIST.RT rows against the
BETA.DIST(ref) rows at the same (x, a, b)."""
import csv, math
from collections import defaultdict
import numpy as np
rows = list(csv.DictReader(open("ix_evidence.tsv"), delimiter="\t"))
by = defaultdict(list)
for r in rows:
    try: e = abs(float(r["err_ulp"]))
    except ValueError: continue
    if math.isfinite(e): by[(r["source_fn"], r["confidence"])].append(e)
print(f"{'source':16s} {'confidence':11s} {'n':>6s} {'=0':>6s} {'<=4':>6s} {'median':>7s} {'p99':>9s}")
for k in sorted(by):
    e = np.array(by[k])
    print(f"{k[0]:16s} {k[1]:11s} {len(e):6d} {(e==0).sum():6d} {(e<=4).sum():6d} {np.median(e):7.2f} {np.percentile(e,99):9.1f}")
ref = {(r["x"], r["a"], r["b"]): r for r in rows if r["source_fn"] == "BETA.DIST(ref)"}
cnt = defaultdict(lambda: [0, 0])
for r in rows:
    if r["source_fn"] != "T.DIST.RT": continue
    k = (r["x"], r["a"], r["b"])
    if k in ref and r["confidence"] in ("direct", "derived"):
        same = float(ref[k]["excel_ix"]) == float(r["excel_ix"])
        cnt[r["branch"]][0] += same; cnt[r["branch"]][1] += 1
print("T.DIST.RT rows vs BETA.DIST(ref) at the same (x,a,b), by bratio branch (same / n):", {k: tuple(v) for k, v in cnt.items()})
