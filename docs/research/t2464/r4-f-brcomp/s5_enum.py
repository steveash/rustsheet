"""Enumerate ratlib chains on large-shape runs: median within-run residual sd (model - Excel, units, free offset) and
whole-set counts. --control CHAIN replaces Excel by that chain's own values (positive control).
Usage: s5_enum.py SET [min max(d)] [--control pw,hl,sum,L]"""
import itertools
import statistics as S
import sys

import fblib as B
import ratlib as RL

argv = [a for a in sys.argv[1:] if not a.startswith("--")]
SET = argv[0] if argv else "a"
MIN = float(argv[1]) if len(argv) > 1 else 40
ctl = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--control=")), None)
rows = [r for r in B.load(SET) if max(r["d1"], r["d2"]) >= MIN]
if ctl:
    pw, hl, sm, L = ctl.split(",")
    for r in rows:
        r["excel"] = RL.value(r["x"], r["d1"], r["d2"], dict(pw=pw, hl=hl, sum=sm, L=L))
runs: dict = {}
for r in rows:
    runs.setdefault(r["blk"], []).append(r)


def run_sd(fn):
    sds = []
    for rr in runs.values():
        u = [B.F.units(fn(r), r["excel"]) for r in rr]
        sds.append(S.pstdev(u))
    return S.median(sds), S.mean(sds)


print(f"# {SET}: {len(rows)} rows in {len(runs)} runs with max(d) >= {MIN}" + (f"; CONTROL {ctl}" if ctl else ""))
med, mean = run_sd(lambda r: r["cr"])
print(f"{'CR':28s} run sd median {med:6.1f} mean {mean:6.1f} | {B.F.fmt(B.F.tally([(r['cr'], r['excel']) for r in rows]))}")
res = []
for pw, hl, sm, L in itertools.product(("x", "r", "rdiv", "mx", "mx1", "exact"), ("st", "ext", "split", "ab"),
                                       ("ext", "st"), ("xl", "ln")):
    o = dict(pw=pw, hl=hl, sum=sm, L=L)
    vals = {id(r): RL.value(r["x"], r["d1"], r["d2"], o) for r in rows}
    med, mean = run_sd(lambda r: vals[id(r)])
    t = B.F.tally([(vals[id(r)], r["excel"]) for r in rows])
    res.append((med, f"{pw},{hl},{sm},{L}", mean, t))
for med, k, mean, t in sorted(res)[:20]:
    print(f"{k:28s} run sd median {med:6.1f} mean {mean:6.1f} | {B.F.fmt(t)}")
