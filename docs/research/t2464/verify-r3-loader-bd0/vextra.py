"""Two side checks on the frozen r3 model.
cut:    per-block exact counts with the series cut at 0.2 against 0.25 (worker's claim 7).
offset: per-block best constant C (all cells; units 2^-53, relative to the shape's first block) by branch pair,
        to check that ss and ll blocks of one shape need different constants (worker's claim 3).
Usage: vextra.py cut|offset CASE_SUFFIX"""
from __future__ import annotations

import sys

import vdata as V
import vmodel as M
import vscore as S


def main(what, suf):
    if what == "cut":
        S.MODELS.clear()
        S.MODELS.update({"r3": {"v": "x", "ph": 1, "ld": "x"}, "cut0.20": {"v": "x", "ph": 1, "ld": "x"},
                         "cut0.25": {"v": "x", "ph": 1, "ld": "x", "cut": 0.25}})
        rows, res, pb, br = S.run([suf])
        for blk in sorted({r["blk"] for r in rows}, key=lambda s: int(s.split("#")[1])):
            i0 = next(i for i, r in enumerate(rows) if r["blk"] == blk)
            a, b = pb[("cut0.20", blk)]["x"], pb[("cut0.25", blk)]["x"]
            if a != b:
                (k, Mm), (nk, Mq) = M.means(rows[i0]["x"], rows[i0]["a"], rows[i0]["b"])
                vp, vq = (k - Mm) / (k + Mm), (nk - Mq) / (nk + Mq)
                print(f"{blk:8s} ({rows[i0]['a']:g},{rows[i0]['b']:g}) vp {vp:+.4f} vq {vq:+.4f}  cut0.20 {a}  cut0.25 {b}"
                      f" of {pb[('cut0.20', blk)]['n']}")
        print({k: v for k, v in res.items() if k[1] == "ALL"})
        return
    rows = V.load_case(f"t2513-{suf}" if suf.startswith("verify") else f"t2513-r3-loader-bd0-{suf}")
    M.LN.need([r for row in rows for r in M.ratios_needed(row["x"], row["a"], row["b"])])
    ref: dict = {}
    for blk, idx in V.blocks(rows).items():
        r0 = rows[idx[0]]
        X = [M.xpart(rows[i]["x"], r0["a"], r0["b"], S.MODELS["r3"]) for i in idx]
        ivs = [S.e_interval(rows[i]["excel"]) for i in idx]
        # reference: median over the block of (exact ln pdf - model X), so C - ref is the branch's value bias
        ex = sorted(float((M.exact_logpdf(rows[i]["x"], r0["a"], r0["b"]) - x[0]) / S.U) for i, x in zip(idx, X))
        lp = M.LD(ex[len(ex) // 2]) * S.U
        base = [float((iv[0] - x[0] - lp) / S.U) for iv, x in zip(ivs, X)]
        c = S.deepest([(b + iv[1], b + iv[2]) for b, iv in zip(base, ivs)])
        print(f"{blk:8s} ({r0['a']:g},{r0['b']:g}) {X[0][1]}  C - median(exact lnpdf - X) {c:+8.2f} units")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
