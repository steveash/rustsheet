"""Re-score r3-tdensity's LT1e and its constant variants with the verifier's own parser and scorer.

usage: vrun.py [--sets old,a,hold,hold2,vhold] [--scan]
  --scan: per-df best constant offset on each probe set (upper bound / look-elsewhere check).
The candidate (tdlib.lt1, Loader dt with LT1 knobs) is imported from ../r3-tdensity after reading it against
RESULTS.md; the offsets are recomputed here from lt1_offsets.json and r3-loader-const's grid_e.tsv.
"""

from __future__ import annotations

import collections
import json
import sys
from pathlib import Path

import numpy as np

import vlib as V

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-tdensity"))
import tdlib as TD  # noqa: E402

LD = np.longdouble


def st(x) -> LD:
    return LD(float(x))  # x87 64-bit result stored to a double


def df1(t: float) -> float:
    return float(st(LD(1) / st(LD(np.pi) * st(LD(1) + st(LD(t) * LD(t))))))


def df2(t: float) -> float:
    return float(TD.exp_v6(st(LD(-1.5) * LD(TD.LN(float(st(LD(2) + st(LD(t) * LD(t)))))))))


def offsets():
    ins = {
        float(k): v
        for k, v in json.loads((HERE.parent / "r3-tdensity" / "lt1_offsets.json").read_text()).items()
    }
    e = TD.stirlerr_e()
    d = {n: e[(n + 1) / 2] - e[n / 2] for n in ins}
    c0 = float(np.mean([ins[n] - d[n] for n in ins]))

    def etab(shift: float = 0.0):
        out = {}
        for n in range(3, 56):
            a, b = (n + 1) / 2 + shift, n / 2 + shift
            if a in e and b in e:
                out[float(n)] = e[a] - e[b] + c0
        return out

    return ins, c0, etab


def sets(rows):
    old = [r for r in rows if not r["case"].startswith("t2513") and r["t"] != 0 and r["excel"] != 0]
    by = collections.defaultdict(list)
    for r in rows:
        by[r["case"]].append(r)
    return {
        "old_df12": [r for r in old if r["n"] < 3],
        "old_df3+": [r for r in old if r["n"] >= 3],
        "a1+a2": by["t2513-r3-tdensity-a1"] + by["t2513-r3-tdensity-a2"],
        "hold": by["t2513-r3-tdensity-hold"],
        "hold2": by["t2513-r3-tdensity-hold2"],
        "vhold": [r for c, v in by.items() if c.startswith("t2513-verify-r3-tdensity") for r in v],
    }


def model(r, off: dict) -> float:
    if r["n"] == 1:
        return df1(r["t"])
    if r["n"] == 2:
        return df2(r["t"])
    return TD.lt1(r["t"], r["n"], off.get(r["n"], 0.0))


def main():
    want = None
    for a in sys.argv[1:]:
        if a.startswith("--sets="):
            want = a.split("=", 1)[1].split(",")
    rows, bad = V.load_all()
    S = sets(rows)
    ins, c0, etab = offsets()
    print(f"parse failures {bad}; c0 = {c0:+.4f}")
    variants = {
        "LT1e": etab(),
        "exactC": {},
        "LT1-insample": ins,
        "constmean": {float(n): float(np.mean(list(ins.values()))) for n in range(3, 400)},
        "etab-shift+1": etab(1.0),
        "etab-shift-1": etab(-1.0),
        "etab-shift+2": etab(2.0),
        "etab-shift+0.5": etab(0.5),
    }
    for nm, rr in S.items():
        if want and nm not in want or not rr:
            continue
        keys = collections.Counter((abs(r["t"]), r["n"]) for r in rr)
        print(
            f"\n== {nm}: n={len(rr)} unique(|t|,n)={len(keys)} dfs={sorted(set(r['n'] for r in rr))[:12]}..."
        )
        print(f"  CR            {V.fmt(V.score((V.cr(r['t'], r['n']), r['excel']) for r in rr))}")
        for vn, off in variants.items():
            if nm == "old_df12" and vn != "LT1e":
                continue
            print(f"  {vn:14s}{V.fmt(V.score((model(r, off), r['excel']) for r in rr))}")
        if nm in ("hold2", "vhold", "hold"):
            per = collections.defaultdict(list)
            for r in rr:
                per[r["n"]].append(r)
            for n in sorted(per):
                q = per[n]
                print(
                    f"   df {n:g}: LT1e {V.fmt(V.score((model(r, variants['LT1e']), r['excel']) for r in q))}"
                    f" | exactC {V.fmt(V.score((model(r, {}), r['excel']) for r in q))}"
                    f" | CR {V.fmt(V.score((V.cr(r['t'], r['n']), r['excel']) for r in q))}"
                    f" | LT1e off {variants['LT1e'].get(n, 0.0):+.2f}"
                )
        if "--scan" in sys.argv and nm in ("hold2", "vhold", "hold"):
            per = collections.defaultdict(list)
            for r in rr:
                per[r["n"]].append(r)
            for n in sorted(per):
                q = per[n]
                grid = np.arange(-4.0, 4.0001, 0.1)
                sc = [V.score((TD.lt1(r["t"], n, float(g)), r["excel"]) for r in q)[1] for g in grid]
                b = int(np.argmax(sc))
                print(
                    f"   scan df {n:g}: best off {grid[b]:+.1f} exact {sc[b]}/{len(q)}; at LT1e off "
                    f"{variants['LT1e'].get(n, 0.0):+.2f}; at 0: {sc[40]}"
                )


if __name__ == "__main__":
    main()
