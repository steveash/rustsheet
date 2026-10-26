"""Verifier step 3: score the same PLAN on the round-2 probe cells (excluded by r2-impact), per probe family, as a
check on data the PLAN's upstream (round-1 pieces, r2-bratio BEST selection) mostly did not see. Writes v3_cells.pkl."""
import csv, math, pickle, re, sys
from collections import Counter, defaultdict
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import v2_plan as V
from v1_offcells import w15


def build_r2():
    status, fails = V.load_engine()
    rows = []
    for r in csv.DictReader(open(V.T / "r2-impact" / "cells_all.tsv"), delimiter="\t"):
        if "t2464-r2-" not in r["case"]:
            continue
        try:
            ex = float(r["excel"])
        except ValueError:
            continue
        m = V.mapping(r["fn"], tuple(V.arg(s) for s in r["args"].split("|")))
        if m is None:
            continue
        st = status.get(r["case"])
        if st is None or st[0] not in ("pass", "fail"):
            continue
        f = fails.get((r["case"], r["cell"]))
        eng = ex if f is None else (float(f["actual"]["v"]) if isinstance(f["actual"], dict) and f["actual"].get("t") == "n" else math.nan)
        rows.append(dict(k=(r["case"], r["cell"]), fn=r["fn"], args=r["args"], m=m, excel=ex, eng=eng, pop="r2", gated=False))
    return rows


def main():
    p = HERE / "v3_cells.pkl"
    rows = pickle.loads(p.read_bytes()) if p.exists() else V.run_models(build_r2())
    p.write_bytes(pickle.dumps(rows))
    g = defaultdict(list)
    for c in rows:
        g[re.sub(r"t2464-(r2-[a-z0-9]+(?:-[a-z]+)?).*", r"\1", c["k"][0].split("/")[-1])].append(c)
    K = ("n", "eng_ex", "eng_vis", "ex", "u4", "w15", "vis", "fixed", "broke", "lost_ex")
    for name, rs in sorted(g.items()) + [("ALL r2", rows)]:
        t = V.tally(rs)
        print(f"{name:28s} " + " ".join(f"{k}={t[k]}" for k in K))


if __name__ == "__main__":
    main()
