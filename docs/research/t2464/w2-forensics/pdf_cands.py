"""Score pdf candidates per block (w2f blocks + beta_exact pdf rows): bit-exact / within 4 ulp / within 15 digits,
and the correlation of the candidate's own error with Excel's error (structure test)."""
import csv, collections, sys
import numpy as np, mpmath
from common import *
from toms import call, rcall
M = mpmath.mpf
def blocks():
    g = collections.defaultdict(list)
    for r in csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"):
        if r["case"] == "t2464-w2f-unit-shape" or r["cum"] != "0": continue
        i = int(r["cell"].split("!A")[1]) - 1
        size = 2048 if "xruns" in r["case"] else 1024
        g[f"{r['case'][10:]}:{i // size}"].append((float(r["x"]), float(r["a"]), float(r["b"]), float(r["excel"]), M(r["exact"])))
    for r in load_beta_exact():
        if r["cum"]: continue
        g["beta_exact:" + r["case"].split("/")[-1].replace(".case.json", "")].append((r["x"], r["a"], r["b"], r["excel"], exact_pdf(r["x"], r["a"], r["b"])))
    return g
def cands(pts):
    q = [(a, b, x, 1 - x) for x, a, b, _, _ in pts]
    out = {"exact": [float(t) for *_, t in pts]}
    out["R3.6 dbeta sse"] = rcall("dbeta", q)
    out["R3.6 dbeta 387"] = rcall("dbeta", q, "rdbeta_387")
    out["toms brcomp/(xy) sse"] = call("dbeta_brc", q)
    out["toms brcomp/(xy) 387"] = call("dbeta_brc", q, "toms_387")
    return out
if __name__ == "__main__":
    g = blocks()
    tot = collections.defaultdict(lambda: np.zeros(4, int))
    for k in sorted(g):
        pts = g[k]
        ex = [e for _, _, _, e, _ in pts]
        line = [f"{k:40s} n={len(pts):4d} a={pts[0][1]:.4g} b={pts[0][2]:.4g}"]
        ee = np.array([float((M(e) - t) / t) / U for *_, e, t in pts])
        for n, vals in cands(pts).items():
            s = score(zip(vals, ex)); tot[n] += np.array(s)
            ce = np.array([float((M(v) - t) / t) / U for v, (*_, t) in zip(vals, pts)])
            r = np.corrcoef(ce, ee)[0, 1] if ce.std() > 0 and ee.std() > 0 else float("nan")
            line.append(f"{n[:22]}: {s[1]}/{s[2]}/{s[3]} r={r:+.2f}")
        print(" | ".join(line))
    print("TOTAL n/exact/w4/w15", {k: tuple(v) for k, v in tot.items()})
