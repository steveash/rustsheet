"""Frozen-model scoring with a free per-block constant: C is fitted on the even (md5) half of each block and the
odd half is scored: bit-exact / within 4 ulp / within 15 digits, by branch pair (p side, q side), against the
correctly rounded value (CR) on the same cells.
Usage: final_score.py SRC [SRC ...]   (SRC: old | a | b | hold)"""
import sys
from collections import defaultdict

import mpmath

import r3lib as L
import scan as S

MODELS = {  # name: (x-part knobs, chain)
    "r2-x87 (C)": ({}, "C"),
    "r2-x87 (Ce)": ({}, "Ce"),
    "r3 (Ce)": ({"v": "x", "ph": 1, "ld": "x"}, "Ce"),
    "r3 (K)": ({"v": "x", "ph": 1, "ld": "x"}, "K"),
}


def tally_add(t, m, e):
    t[0] += 1
    t[1] += m == e
    t[2] += L.DL.ulpd(m, e) <= 4
    t[3] += L.DL.within15(m, e)


def predict(row, b1, b2, chain, C0, off, L5, h):
    C = L.LDB(C0) + L.LDB(off) * L.U53
    ab1 = None
    return L.chain_out(chain, C, b1, b2, L5, h, ab1)


def main(srcs):
    for src in srcs:
        rows = S.load(src)
        if src == "old":
            rows = [r for r in rows if r["blk"].startswith("xruns")]
        res = defaultdict(lambda: [0, 0, 0, 0])
        for blk, idx in sorted(L.blocks(rows).items()):
            if len(idx) < 64:
                continue
            od = [i for i in idx if rows[i]["hold"]]
            br = L.xpart(rows[idx[0]]["x"], rows[idx[0]]["a"], rows[idx[0]]["b"], {"ld": "x"})[2]
            for i in od:
                r = rows[i]
                tally_add(res[("CR", br)], float(mpmath.exp(L.exact_logpdf(r["x"], r["a"], r["b"]))), r["excel"])
            for name, (o, chain) in MODELS.items():
                f = L.fit_block(rows, idx, o, chain, W=1024.0)
                L5, h = L.shape_pieces(rows[idx[0]]["a"], rows[idx[0]]["b"])
                for i in od:
                    b1, b2, _ = f["X"][i]
                    m = predict(rows[i], b1, b2, chain, f["C0"], f["off"], L5, h)
                    tally_add(res[(name, br)], m, rows[i]["excel"])
            L.save_cache()
        print(f"== {src} (odd half of every block; C fitted on the even half)")
        brs = sorted({b for _, b in res})
        for name in ["CR"] + list(MODELS):
            tot = [0, 0, 0, 0]
            parts = []
            for b in brs:
                t = res[(name, b)]
                tot = [p + q for p, q in zip(tot, t)]
                parts.append(f"{b}: {t[1]}/{t[2]}/{t[3]} of {t[0]}")
            print(f"  {name:12s} ALL {tot[1]}/{tot[2]}/{tot[3]} of {tot[0]}   " + "   ".join(parts))


if __name__ == "__main__":
    main(sys.argv[1:])
