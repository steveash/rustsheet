"""Chain-free structure checks, own code.

1. BINOM vs BETA shared rounding (claim 2): per run (same (k,n), consecutive p), the exponent error
   e = (ln excel - ln exact) * 2^53 of BINOM.DIST(k,n,p) and of BETA.DIST(p,k+1,n-k+1); sd of each, and sd of
   their difference.  Shared p-dependent roundings => sd(diff) at the output-rounding floor (about 0.5-1).
   NEG at the same p (hold, v) is the negative control: it is a different (shifted) kernel.
2. Admissible stored E (claim 5): for interior BINOM cells with |ln y| >= 4, is there a double E with
   Excel EXP(E) == y?  If E were a stored double, every cell would be admissible.
Usage: vstruct.py <a|hold|v>"""

from __future__ import annotations

import collections
import math
import random
import statistics
import sys

import mpmath

import vdata as D
import vlib as V
import vmodels as M


def err_units(r) -> float:
    ex = V.exact(r["kern"], r["k"], r["n"], r["p"])
    return float((mpmath.log(r["excel"]) - mpmath.log(ex)) * mpmath.mpf(2) ** 53)


def run_id(r):
    return (r["k"], r["n"], round(math.log(float(r["p"])), 3))


def share(name: str) -> None:
    rows = [r for r in D.load(name) if r["excel"] > 0]
    by = collections.defaultdict(dict)
    for r in rows:
        if r["kern"] == "NEG":  # NEGBINOM(n-k, k, p) sits at the row of BINOM(k, n, p): f = n-k, r = k
            k, n = r["n"], r["k"] + r["n"]
        else:
            k, n = r["k"], r["n"]
        if 0 < k < n:
            by[(k, n, float(r["p"]))][r["kern"]] = err_units(r)
    runs = collections.defaultdict(list)
    for (k, n, p), d in by.items():
        runs[(k, n, round(math.log(p), 2))].append(d)
    print(f"== {name}: {len(runs)} runs; sd(BIN) sd(BETA) sd(BIN-BETA) [sd(NEG) sd(BIN-NEG)]")
    dsd, ratios = [], []
    for key in sorted(runs):
        ds = [d for d in runs[key] if "BIN" in d and "BETA" in d]
        if len(ds) < 8:
            continue
        b = [d["BIN"] for d in ds]
        be = [d["BETA"] for d in ds]
        df = [x - y for x, y in zip(b, be)]
        s = f"   {key!s:28s} n={len(ds):3d} {statistics.pstdev(b):8.2f} {statistics.pstdev(be):8.2f} {statistics.pstdev(df):6.2f}"
        dn = [d for d in ds if "NEG" in d]
        if len(dn) >= 8:
            s += f"   {statistics.pstdev([d['NEG'] for d in dn]):8.2f} {statistics.pstdev([d['BIN'] - d['NEG'] for d in dn]):8.2f}"
        print(s)
        dsd.append(statistics.pstdev(df))
        ratios.append(statistics.pstdev(b))
    if dsd:
        print(
            f"   sd(BIN-BETA): min {min(dsd):.2f} max {max(dsd):.2f} over {len(dsd)} runs; sd(BIN) max {max(ratios):.1f}"
        )


def admissible(name: str, sample: int = 400) -> None:
    rows = [r for r in D.load(name) if r["kern"] == "BIN" and 0 < r["k"] < r["n"] and r["excel"] > 0]
    rows = [r for r in rows if abs(math.log(r["excel"])) >= 4]
    random.Random(7).shuffle(rows)
    rows = rows[:sample]
    W = 48
    cand = []
    for r in rows:
        e0 = float(mpmath.log(r["excel"]))
        es = [e0]
        lo = hi = e0
        for _ in range(W):
            lo, hi = math.nextafter(lo, -math.inf), math.nextafter(hi, math.inf)
            es += [lo, hi]
        cand.append(es)
    vals = M.xl("EXP", [e for es in cand for e in es])
    ok = edge = 0
    j = 0
    for r, es in zip(rows, cand):
        got = vals[j : j + len(es)]
        j += len(es)
        hit = [e for e, v in zip(es, got) if v == r["excel"]]
        ok += bool(hit)
        edge += bool(hit) and (min(hit) == min(es) or max(hit) == max(es))
    print(
        f"== {name}: interior BINOM cells with |ln y| >= 4 that are EXP of some double: {ok}/{len(rows)} "
        f"(window +-{W} doubles; {edge} at the window edge)"
    )


if __name__ == "__main__":
    share(sys.argv[1])
    admissible(sys.argv[1])
