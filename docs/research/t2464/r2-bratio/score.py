"""Score the bratio port on every cdf data set, as an attribution ladder (one switch at a time), per branch.

usage: score.py [ladder|<config names...>]   writes results.pkl (values per config) and prints markdown tables.
"""
import pickle
import sys
from dataclasses import replace

import bratio_excel as bx
import score_lib as sl

T = bx.TOMS_XL
LADDER = {
    "R_LIKE": bx.R_LIKE,
    "TOMS_XL": T,
    "+closed": replace(T, closed=True),
    "+w3bpser(z53)": replace(T, closed=True, bpser="w3", w3_z="z53", w3_lg="acc"),
    "+z64": replace(T, closed=True, bpser="w3", w3_z="z64", w3_lg="acc"),
    "+lg_toms": replace(T, closed=True, bpser="w3", w3_z="z64", w3_lg="toms"),
    "+pdf_brcmp1": replace(T, closed=True, bpser="w3", w3_z="z64", w3_lg="toms", brcmp1="pdf"),
    "+w4_bup": replace(T, closed=True, bpser="w3", w3_z="z64", w3_lg="toms", brcmp1="pdf", bup_arith="w4"),
    "+pdf_brcomp(x29=0.29)": replace(bx.BEST, x29=0.29),
    "BEST": bx.BEST,
}


def main():
    names = sys.argv[1:] or list(LADDER)
    rows = sl.load()
    pts = sorted({(r["a"], r["b"], r["x"], r["y"]) for r in rows})
    res = {}
    for n in names:
        out = dict(zip(pts, bx.run(pts, LADDER[n])))
        res[n] = [out[(r["a"], r["b"], r["x"], r["y"])] for r in rows]
        print("done", n, file=sys.stderr)
    pickle.dump((rows, res), open(sl.HERE / "results.pkl", "wb"))
    report(rows, res)


def readme_branch(r):
    if r["set"] == "ix":
        return "ix"
    return "direct" if r["x"] <= r["a"] / (r["a"] + r["b"]) else "reflected"


def report(rows, res, names=None):
    names = names or list(res)
    vals = {n: [sl.side_value(r, v) for r, v in zip(rows, res[n])] for n in names}
    last = names[-1]
    br = [v[2] for v in res[last]]
    for i, r in enumerate(rows):
        r["branch"] = br[i]
    sl.table(rows, vals, lambda r: r["branch"], f"per bratio branch (branch as fired in {last}), all sets")
    sl.table(rows, vals, lambda r: (r["set"], readme_branch(r)), "per data set and README branch")
    sl.table(rows, vals, lambda r: (r["set"], r["branch"]), "per data set and bratio branch", holdout=False)


if __name__ == "__main__":
    main()
