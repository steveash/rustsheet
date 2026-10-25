"""a3: whole-model scores (exact / 4 ulp / 15 digits) of F density candidates with exact constants, per df class,
on cells.tsv and r3-tdensity's d1 = 1 column; also the within-(d1,d2) sd of (model - Excel) in units.
Usage: a3_score.py [model ...]"""
import collections
import statistics as st
import sys

import a2_struct as A2
import flib as F

MODELS = {k: v for k, v in A2.VARIANTS.items()}


def cls(r):
    m, n = r["d1"], r["d2"]
    if m == 2:
        return "d1=2"
    if m == 1:
        return "d1=1"
    return "d2<=2" if n <= 2 else "d1,d2>2"


def main():
    rows = F.with_exact(F.cells_rows() + F.tdens_rows())
    names = sys.argv[1:] or ["CR"] + list(MODELS)
    for nm in names:
        g = collections.defaultdict(list)
        dev = collections.defaultdict(list)
        for r in rows:
            v = r["cr"] if nm == "CR" else F.df_value(r["x"], r["d1"], r["d2"], MODELS[nm])
            key = (r["src"].split("-")[0], cls(r))
            g[key].append((v, r["excel"]))
            g[("all",)].append((v, r["excel"]))
            dev[(r["d1"], r["d2"])].append(F.units(v, r["excel"]))
        sds = [st.pstdev(v) for v in dev.values() if len(v) >= 8]
        print(f"{nm:10s} " + " | ".join(f"{'/'.join(k)} {F.fmt(F.tally(v))}" for k, v in sorted(g.items()))
              + f" | median within-shape sd {st.median(sds):.2f}")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
