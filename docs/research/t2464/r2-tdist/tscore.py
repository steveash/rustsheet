"""Score T tail cells under (argument variant x kernel config) specs, per bratio branch and t side.

usage: tscore.py [--only BRANCHSUB] spec [spec ...]   spec = "args=yz,alnrel=xl,erfc=xl,..." (XCfg fields; BEST base)
The branch column is the branch fired under the first spec. Upper-side cells (1 - L) are excluded (see compose.py).
"""
import ast
import statistics
import sys
from collections import defaultdict

import tails
import tlib
import xlk


def parse(spec):
    kw, args = {}, "yz"
    for item in filter(None, spec.split(",")):
        k, v = item.split("=")
        if k == "args":
            args = v
            continue
        try:
            kw[k] = ast.literal_eval(v)
        except (ValueError, SyntaxError):
            kw[k] = v
    return args, xlk.XCfg(**kw)


def rows_L():
    rows = [r for r in tlib.load(("rt", "2t", "cdf")) if r["t"] != 0.0 and r["excel"] != 0.0
            and not ((r["kind"] == "cdf") == (r["t"] > 0) and r["kind"] != "2t")]
    for r in rows:
        r["cr"] = tlib.exact_cell(r)
    tlib.save_exact()
    return rows


def evaluate(rows, spec):
    args, cfg = parse(spec)
    pts = [tails.kargs(args, abs(r["t"]), r["n"]) for r in rows]
    uniq = sorted({p for p, _ in pts})
    res = dict(zip(uniq, xlk.run(uniq, cfg)))
    vals, brs = [], []
    for r, (p, flipped) in zip(rows, pts):
        w, w1, b = res[p]
        I = w1 if flipped else w
        vals.append(I if r["kind"] == "2t" else I / 2)
        brs.append(b)
    return vals, brs


def band(n):
    for lim in (2, 10, 30, 100, 1000):
        if n <= lim:
            return f"n<={lim}"
    return "n>1000"


def main():
    argv = sys.argv[1:]
    only = None
    if argv and argv[0] == "--only":
        only, argv = argv[1], argv[2:]
    rows = rows_L()
    out = {s: evaluate(rows, s) for s in argv}
    br = out[argv[0]][1]
    g = defaultdict(list)
    for i, r in enumerate(rows):
        if only and only not in br[i]:
            continue
        g[(br[i], "t<=1" if abs(r["t"]) <= 1 else "t>1", band(r["n"]))].append(i)
        g[(br[i], "all", "")].append(i)
        g[("ALL", "", "")].append(i)
    print("specs:", " ; ".join(f"[{k}] {s}" for k, s in enumerate(argv)))
    for k in sorted(g, key=str):
        idx = g[k]
        hidx = [i for i in idx if rows[i]["hold"]]
        cells = []
        for s in argv + ["CR"]:
            v = out[s][0] if s != "CR" else [r["cr"] for r in rows]
            n, ex, f, w = tlib.TALLY((v[i], rows[i]["excel"]) for i in idx)
            h = tlib.TALLY((v[i], rows[i]["excel"]) for i in hidx)[1]
            e = [(v[i] - rows[i]["excel"]) / abs(rows[i]["excel"]) / 2**-53 for i in idx]
            md = statistics.median(e)
            mad = statistics.median(abs(q - md) for q in e)
            cells.append(f"{ex}/{f}/{w} h{h} med{md:+.1f} mad{mad:.1f}")
        print(f"{k} n={len(idx)} h={len(hidx)} | " + " | ".join(cells))


if __name__ == "__main__":
    main()
