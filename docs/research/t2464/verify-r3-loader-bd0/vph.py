"""Chain-free check of the +delta(2kv) claim: per series-side cell, c_i = midpoint of Excel's admissible extended E
minus the model's X without the ph term (r3-ph), in units of 2^-53.  If Excel's X carries +delta, with
delta = fl(2k v) - 2k v per series side, c_i - median = slope * delta_sum + noise with slope +1.
Placebos: delta of fl(2 M v) (the other argument) and delta shuffled within the block.
Usage: vph.py CASE_SUFFIX [...]"""
from __future__ import annotations

import sys

import numpy as np

import vdata as V
import vmodel as M
import vscore as S


def deltas(x, a, b, which="k"):
    out = M.LD(0)
    for k, Mm in M.means(x, a, b):
        if M.is_series(k, Mm):
            K, Mm_ = M.LD(k), M.LD(Mm)
            d = M.st(K - Mm_)
            v = M.st(d / M.st(K + Mm_))
            ej = M.LD(2) * (K if which == "k" else Mm_) * v
            out += M.st(ej) - ej
    return float(out / S.U)


def main(sufs):
    rows = []
    for s in sufs:
        rows += V.load_case(f"t2513-{s}" if s.startswith("verify") else f"t2513-r3-loader-bd0-{s}")
    M.LN.need([r for row in rows for r in M.ratios_needed(row["x"], row["a"], row["b"])])
    rng = np.random.default_rng(1)
    tot = {"k": [], "M": [], "shuf": []}
    for blk, idx in V.blocks(rows).items():
        r0 = rows[idx[0]]
        X = [M.xpart(rows[i]["x"], r0["a"], r0["b"], S.MODELS["r3-ph"]) for i in idx]
        if "s" not in X[0][1]:
            continue
        ivs = [S.e_interval(rows[i]["excel"]) for i in idx]
        ok = [j for j, iv in enumerate(ivs) if iv]
        c = np.array([float((ivs[j][0] - X[j][0]) / S.U) for j in ok]) + np.array([(ivs[j][1] + ivs[j][2]) / 2 for j in ok])
        c -= np.median(c)
        dk = np.array([deltas(rows[idx[j]]["x"], r0["a"], r0["b"], "k") for j in ok])
        dM = np.array([deltas(rows[idx[j]]["x"], r0["a"], r0["b"], "M") for j in ok])
        ds = rng.permutation(dk)
        line = []
        for name, d in (("k", dk), ("M", dM), ("shuf", ds)):
            if d.std() == 0:
                line.append(f"{name}: const")
                continue
            sl = np.polyfit(d, c, 1)[0]
            rho = np.corrcoef(d, c)[0, 1]
            tot[name].append(rho)
            line.append(f"{name}: slope {sl:+.2f} rho {rho:+.2f}")
        print(f"{blk:8s} ({r0['a']:g},{r0['b']:g}) {X[0][1]} sd(c) {c.std():6.2f} sd(dk) {dk.std():5.2f}  " + "  ".join(line))
    for name, v in tot.items():
        v = np.array(v)
        print(f"SUMMARY {name}: blocks {len(v)}  median rho {np.median(v):+.3f}  rho>0 in {int((v > 0).sum())}")


if __name__ == "__main__":
    main(sys.argv[1:])
