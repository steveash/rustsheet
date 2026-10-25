"""Quick experiment: score config variants on a subset, per block.
usage: exp.py <set|*> <branch-substring|*> <base config: BEST|TOMS_XL|...> key=value[,key=value] ...
Each positional variant is a comma-separated list of Cfg overrides applied to the base (values parsed as Python
literals when possible). Prints exact/w4/w15 and the median/sd of the error in units of 2^-53 per block."""
import ast
import statistics
import sys
from collections import defaultdict
from dataclasses import replace

import bratio_excel as bx
import score_lib as sl


def parse(spec, base):
    kw = {}
    for item in filter(None, spec.split(",")):
        k, v = item.split("=")
        try:
            kw[k] = ast.literal_eval(v)
        except (ValueError, SyntaxError):
            kw[k] = v
    return replace(base, **kw)


def main():
    want_set, want_br, base = sys.argv[1], sys.argv[2], getattr(bx, sys.argv[3])
    variants = {"base": base}
    variants.update({v: parse(v, base) for v in sys.argv[4:]})
    rows = [r for r in sl.load() if want_set in ("*", r["set"])]
    br = bx.run([(r["a"], r["b"], r["x"], r["y"]) for r in rows], base)
    keep = [i for i, b in enumerate(br) if want_br == "*" or want_br in b[2]]
    rows = [rows[i] for i in keep]
    pts = [(r["a"], r["b"], r["x"], r["y"]) for r in rows]
    vals = {n: [sl.side_value(r, v) for r, v in zip(rows, bx.run(pts, c))] for n, c in variants.items()}
    vals["CR"] = [r["cr"] for r in rows]
    g = defaultdict(list)
    for i, r in enumerate(rows):
        g[(r["set"], r["blk"], round(r["a"], 3), round(r["b"], 3), br[keep[i]][2])].append(i)
    g[("ALL",)] = list(range(len(rows)))
    for k, idx in sorted(g.items(), key=str):
        out = [f"{k} n={len(idx)}"]
        for n, v in vals.items():
            n_, ex, f, w = sl.tally((v[i], rows[i]["excel"]) for i in idx)
            e = [(v[i] - rows[i]["excel"]) / abs(rows[i]["excel"]) / 2**-53 for i in idx if rows[i]["excel"]]
            out.append(f"{n}: {ex}/{f}/{w} med {statistics.median(e):+.1f} sd {statistics.pstdev(e):.1f}")
        print(" | ".join(out))


if __name__ == "__main__":
    main()
