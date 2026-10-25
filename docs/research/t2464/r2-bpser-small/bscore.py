"""Score bpser variants on the bpser-region cells (min(a,b) <= 1, w_bpser / w1_bpser; plus bpser(a,b>1) as a side
set), per data set, per TOMS bpser sub-branch, tune/hold-out.
usage: bscore.py [sets=be,w2f,w4p1,ix,x29,shapes,runs,tiny] [variants=NAME,...] [--blk]
Writes results_<sets>.pkl (rows + per-variant values) for later analysis (git-ignored)."""

from __future__ import annotations

import pickle
import sys
from collections import defaultdict

import lib
from lib import V

VARIANTS = {
    "W3z64": V(),  # = r2-bratio BEST
    "W3z53": V(z="z53"),
    "TOMS": V(pre="toms", ser="toms"),
    "TOMSpre+W3ser": V(pre="toms", ser="w3"),
    "W3pre+TOMSser": V(pre="w3", ser="toms"),
    "TOMS z64": V(pre="toms", ser="toms", pw="explnx", tomsz="z64"),
    "TOMSpre z64+W3ser": V(pre="toms", ser="w3", pw="explnx", tomsz="z64"),
    "NOFA": V(nofpser=True, noapser=True),  # BEST without fpser / apser
    "NOFA lg=gl": V(nofpser=True, noapser=True, lg="gl"),
    "NOFA lg=toms53": V(nofpser=True, noapser=True, lg="toms53"),
    "NOFA lg=acc": V(nofpser=True, noapser=True, lg="acc"),
    "NOFA z53": V(nofpser=True, noapser=True, z="z53"),
    "L53": V(nofpser=True, noapser=True, lg="L53"),
    "L53/algdiv64": V(nofpser=True, noapser=True, lg="L53/algdiv64"),
    "L53/algdiv53": V(nofpser=True, noapser=True, lg="L53/algdiv53"),
    "L53/glalgdiv": V(nofpser=True, noapser=True, lg="L53/glalgdiv"),
    "L53 eext": V(nofpser=True, noapser=True, lg="L53", eext=True),
    "L53/glalgdiv eext": V(nofpser=True, noapser=True, lg="L53/glalgdiv", eext=True),
    "gl eext": V(nofpser=True, noapser=True, lg="gl", eext=True),
    "GL53": V(nofpser=True, noapser=True, lg="GL53"),
    "GL53/glalgdiv": V(nofpser=True, noapser=True, lg="GL53/glalgdiv"),
    "BEST+GL53/glalgdiv": V(lg="GL53/glalgdiv"),  # with TOMS fpser/apser kept
    "L53 alx64": V(nofpser=True, noapser=True, lg="L53", alx="64"),
    "L53 alxln64": V(nofpser=True, noapser=True, lg="L53", alx="ln64"),
}


def args():
    kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
    sets = kw.get("sets", "be,w2f,w4p1,ix,x29,shapes,runs,tiny").split(",")
    names = kw.get("variants", ",".join(VARIANTS)).split(",")
    return sets, names


def load(sets):
    old = [s for s in sets if s in ("be", "w2f", "w4p1", "ix", "x29")]
    new = [s for s in sets if s in lib.PROBES]
    rows = (lib.sl.load(tuple(old)) if old else []) + lib.load_probe(tuple(new))
    for r in rows:
        if r["set"] in lib.PROBES:
            continue
        r["hold"] = lib.shape_hold(r["a"], r["b"])  # shape-level hold-out everywhere
    return rows


def main():
    sets, names = args()
    rows = load(sets)
    pts = [(r["a"], r["b"], r["x"], r["y"]) for r in rows]
    vals, labs = {}, None
    for nm in names:
        res = lib.run(pts, VARIANTS[nm])
        vals[nm] = [lib.sl.side_value(r, q[:3]) for r, q in zip(rows, res)]
        if labs is None:
            labs = [lib.label(q) for q in res]
    keep = [i for i, lb in enumerate(labs) if "bpser" in lb or "apser" in lb or "fpser" in lb]
    print(f"rows {len(rows)}, series cells {len(keep)}")
    pickle.dump(dict(rows=rows, labs=labs, vals=vals), open(lib.HERE / f"results_{'_'.join(sets)}.pkl", "wb"))
    groups = defaultdict(list)
    for i in keep:
        r = rows[i]
        br = labs[i].replace("/sw", "")
        groups[(br, r["set"])].append(i)
        groups[(br, "ALL")].append(i)
        if "--blk" in sys.argv:
            groups[(br, r["set"] + ":" + r["blk"])].append(i)
    print(
        "| branch | set | n | " + " | ".join(names) + " | CR |  hold-out exact: " + " ".join(names) + " CR |"
    )
    for g in sorted(groups):
        idx = groups[g]
        cells = [
            lib.fmt(lib.tally((vals[m][i], rows[i]["excel"]) for i in idx)).split(" of")[0] for m in names
        ]
        cr = lib.fmt(lib.tally((rows[i]["cr"], rows[i]["excel"]) for i in idx)).split(" of")[0]
        hidx = [i for i in idx if rows[i]["hold"]]
        hs = [str(lib.tally((vals[m][i], rows[i]["excel"]) for i in hidx)[1]) for m in names]
        hc = lib.tally((rows[i]["cr"], rows[i]["excel"]) for i in hidx)[1]
        print(
            f"| {g[0]} | {g[1]} | {len(idx)} | "
            + " | ".join(cells)
            + f" | {cr} | {'/'.join(hs)}/{hc} of {len(hidx)} |"
        )


if __name__ == "__main__":
    main()
