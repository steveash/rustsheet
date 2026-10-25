"""Re-score r3-binom-pmf's claims with the verifier's own parser and scorer.

Usage: vscore.py <old|a|n|hold|v> [models comma]   (models: CR, EDGE (own closed forms), and blib.MODELS names)
Prints exact / w4 / w15 per region, plus a distinct-(kernel,k,n,p) recount and the R-edge rival where it applies.
Worker model values are cached in wm_<set>.pkl (git-ignored)."""

from __future__ import annotations

import collections
import pickle
import sys
from pathlib import Path

import vdata as D
import vlib as V
import vmodels as M

HERE = Path(__file__).parent
WORKER = ["L-lcx/LF1", "L-lcxx/LF0", "L-ext/LF0"]


def worker_values(name: str, rows, models):
    cache = HERE / f"wm_{name}.pkl"
    got = pickle.loads(cache.read_bytes()) if cache.exists() else {}
    for m in models:
        for r in rows:
            kk = (m, r["cell"], r["case"])
            if kk not in got:
                got[kk] = M.worker_model(m, r)
    cache.write_bytes(pickle.dumps(got))
    return {m: [got[(m, r["cell"], r["case"])] for r in rows] for m in models}


def main() -> None:
    name = sys.argv[1]
    models = sys.argv[2].split(",") if len(sys.argv) > 2 else WORKER
    rows = D.load(name)
    crs = [V.cr(r) for r in rows]
    edge_rows = [i for i, r in enumerate(rows) if r["kern"] == "BIN" and r["k"] in (0, r["n"])]
    edge = dict(zip(edge_rows, M.edge_values([rows[i] for i in edge_rows])))
    wv = worker_values(name, rows, models)
    cols = {"CR": crs, **wv}
    # the claimed edge closed forms replace the worker's for k = 0 / k = n: score them separately as EDGE
    cols["EDGE"] = [edge.get(i) for i in range(len(rows))]
    groups = collections.defaultdict(list)
    for i, r in enumerate(rows):
        groups[D.region(r)].append(i)
        groups["all"].append(i)
    seen, first = set(), []
    for i, r in enumerate(rows):
        if D.key(r) not in seen:
            seen.add(D.key(r))
            first.append(i)
    print(f"== {name}: {len(rows)} rows, {len(first)} distinct (kernel,k,n,p)")
    for g in sorted(groups):
        idx = groups[g]
        print(f"-- {g} ({len(idx)})")
        for c, vals in cols.items():
            if c == "EDGE" and not g.endswith(("k=0", "k=n")):
                continue
            print(f"   {c:12s} {V.fmt(V.score((vals[i], rows[i]['excel']) for i in idx))}")
    # distinct recount of the overall columns
    print("-- all, distinct cells only")
    for c, vals in cols.items():
        if c != "EDGE":
            print(f"   {c:12s} {V.fmt(V.score((vals[i], rows[i]['excel']) for i in first))}")
    # edges: the R dbinom_raw rival branch
    riv = [(i, M.r_edge(rows[i]["k"], rows[i]["n"], rows[i]["p"])) for i in edge_rows]
    riv = [(i, v) for i, v in riv if v is not None]
    if riv:
        print(
            f"-- edge cells where R's dbinom_raw uses bd0 ({len(riv)}; distinct "
            f"{len({D.key(rows[i]) for i, _ in riv})})"
        )
        print(f"   EDGE (claim) {V.fmt(V.score((edge[i], rows[i]['excel']) for i, _ in riv))}")
        print(f"   R-bd0 rival  {V.fmt(V.score((v, rows[i]['excel']) for i, v in riv))}")
        print(f"   CR           {V.fmt(V.score((crs[i], rows[i]['excel']) for i, _ in riv))}")
        print(f"   claim == rival value on {sum(edge[i] == v for i, v in riv)}")
        sh = collections.Counter((rows[i]["n"], float(rows[i]["p"])) for i, _ in riv)
        print(f"   distinct (n, p): {len(sh)}; p values {sorted({p for _, p in sh})}")
    for i in edge_rows:
        if edge[i] != rows[i]["excel"]:
            r = rows[i]
            print("   EDGE miss", r["case"], r["cell"], r["k"], r["n"], float(r["p"]), edge[i], r["excel"])


if __name__ == "__main__":
    main()
