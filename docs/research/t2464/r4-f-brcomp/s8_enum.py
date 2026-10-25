"""Enumerate the grouped chains (ratlib grp/pw/hl/est) by median within-run residual sd (free offset per run).
Usage: s8_enum.py SET [min max(d)] [--control=pw,hl,grp,est]"""
import itertools
import statistics as S
import sys

import fblib as B
import ratlib as RL

argv = [a for a in sys.argv[1:] if not a.startswith("--")]
SET = argv[0] if argv else "big"
MIN = float(argv[1]) if len(argv) > 1 else 0
ctl = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--control=")), None)
KEYS = ("pw", "hl", "grp", "est")
rows = [r for r in B.load(SET) if max(r["d1"], r["d2"]) >= MIN]
if ctl:
    oc = dict(zip(KEYS, ctl.split(",")))
    for r in rows:
        r["excel"] = RL.value(r["x"], r["d1"], r["d2"], oc)
runs: dict = {}
for r in rows:
    runs.setdefault(r["blk"], []).append(r)


def run_sd(vals):
    sds = [S.pstdev([B.F.units(vals[id(r)], r["excel"]) for r in rr]) for rr in runs.values()]
    return S.median(sds), S.mean(sds)


print(f"# {SET}: {len(rows)} rows, {len(runs)} runs, max(d) >= {MIN}" + (f"; CONTROL {ctl}" if ctl else ""))
med, mean = run_sd({id(r): r["cr"] for r in rows})
print(f"{'CR':26s} median {med:6.1f} mean {mean:6.1f}")
res = []
for combo in itertools.product(("x", "xd", "xn", "r", "rdiv", "mx", "exact"), ("st", "ext", "split"),
                               ("kl", "kl2", "none"), ("ext", "st")):
    o = dict(zip(KEYS, combo), L="xl")
    vals = {id(r): RL.value(r["x"], r["d1"], r["d2"], o) for r in rows}
    med, mean = run_sd(vals)
    res.append((med, ",".join(combo), mean, B.F.tally([(vals[id(r)], r["excel"]) for r in rows])))
for med, k, mean, t in sorted(res)[:25]:
    print(f"{k:26s} median {med:6.1f} mean {mean:6.1f} | {B.F.fmt(t)}")
