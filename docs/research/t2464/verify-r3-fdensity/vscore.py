"""Re-score CR and the worker's frozen ratio variants on every set, with the verifier's own parser and scorer.
Also reports duplicated (x, d1, d2) keys within and across sets.
usage: vscore.py SET [variant ...]    SET: a | sw | hold | holdsw | cells | vhold | all"""
import collections
import sys

import vlib as V

SETS = {
    "a": ["t2513-r3-fdensity-a"],
    "sw": ["t2513-r3-fdensity-sw"],
    "hold": ["t2513-r3-fdensity-hold"],
    "holdsw": ["t2513-r3-fdensity-holdsw"],
    "cells": ["t2220-f-dist-sweep-pdf", "t2220-f-dist-sweep-d1-1", "t2220-f-dist-sweep-d2-1", "m9-f-dist",
              "t1049-fdist", "t1530-f-df-cut"],
    "vhold": ["t2513-verify-r3-fdensity-hold"],
}
DEFAULT = ["CR", "ratio-xl", "ratio-xl-e", "ratio-xl-lxm", "ratio-m", "ratio-ln"]


def rows_of(s):
    st = {}
    rows = [r for c in SETS[s] for r in V.case_rows(c, st)]
    return rows, st


def cls(r):
    m, n = r["d1"], r["d2"]
    return "min(d)<=2" if min(m, n) <= 2 else ("max(d)>=50" if max(m, n) >= 50 else "other")


def score(s, names):
    rows, st = rows_of(s)
    keys = collections.Counter((r["x"], r["d1"], r["d2"]) for r in rows)
    dup = sum(v - 1 for v in keys.values() if v > 1)
    print(f"# {s}: {len(rows)} cells, {dup} duplicate keys, skipped {st}")
    for r in rows:
        r["cr"] = float(V.f_exact(r["x"], r["d1"], r["d2"]))
    V.save()
    for nm in names:
        f = None if nm == "CR" else V.model(nm)
        g = collections.defaultdict(list)
        for r in rows:
            v = r["cr"] if f is None else f(r["x"], r["d1"], r["d2"])
            g["all"].append((v, r["excel"]))
            g[cls(r)].append((v, r["excel"]))
        print(f"{s:6s} {nm:13s} " + " | ".join(f"{k} {V.fmt(V.tally(p))}" for k, p in sorted(g.items())),
              flush=True)
    return rows


def main():
    s = sys.argv[1]
    names = sys.argv[2:] or DEFAULT
    todo = [k for k in SETS if k != "vhold"] if s == "all" else [s]
    allkeys = collections.Counter()
    for k in todo:
        rows = score(k, names)
        allkeys.update({(r["x"], r["d1"], r["d2"]) for r in rows})
    if len(todo) > 1:
        print("# keys shared across sets:", sum(1 for v in allkeys.values() if v > 1))


if __name__ == "__main__":
    main()
