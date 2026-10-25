"""Re-score model M2 (own implementation, vlib) on the old pdf cells, w1 probe1, the worker's probes a / hold and
this verifier's own probe(s). Prints data-integrity checks, then bit-exact / within 4 ulp / within 15 digits per
dataset and lnB region, with measured or engine GAMMALN, and TOMS-d or the accurate algdiv.
Usage: vscore.py [dataset ...]   (default: old probe1 a hold vhold)"""

from __future__ import annotations

import collections
import sys

import vlib as V

DATASETS = {
    "old": V.rows_old,
    "probe1": V.rows_probe1,
    "a": lambda: V.rows_case("t2513-r3-m1-residual-a", tag="a"),
    "hold": lambda: V.rows_case("t2513-r3-m1-residual-hold", tag="hold"),
    "vhold": lambda: V.rows_case("t2513-verify-r3-m1-residual-hold", tag="vhold"),
}
GL_CASES = ("t2513-r3-m1-residual-gl", "t2513-verify-r3-m1-residual-hold")


def load(name):
    try:
        rows = DATASETS[name]()
    except FileNotFoundError:
        return []
    return [r for r in rows if min(r["a"], r["b"]) <= 1.0 and 0.0 < r["x"] < 1.0]


def integrity(name, rows):
    bad_echo = sum(r["echo_ok"] is False for r in rows)
    tsv_bad = sum(1 for r in rows if "tsv_args" in r and r["tsv_args"] != (r["x"], r["a"], r["b"]))
    dup = collections.Counter((r["x"], r["a"], r["b"]) for r in rows)
    ndup = sum(c - 1 for c in dup.values() if c > 1)
    shapes = len({(r["a"], r["b"]) for r in rows})
    print(
        f"[{name}] rows {len(rows)}, shapes {shapes}, echo mismatches {bad_echo}, tsv-arg disagreements {tsv_bad}, "
        f"duplicate (x,a,b) {ndup}"
    )
    return {k for k, c in dup.items()}


def report(name, rows, label, lnb_of, ar="d"):
    mod = V.m2(rows, lnb_of, ar)
    by = collections.defaultdict(list)
    for r, m in zip(rows, mod):
        for g in (V.region(r["a"], r["b"]), "ALL"):
            by[g].append((m, r["excel"]))
    for g in ("ALL", "min1", "gl", "algdiv"):
        if by[g]:
            print(f"  {name:7s} {label:22s} {g:7s} {V.fmt(V.score(*zip(*by[g])))}")
    return mod


def main(names):
    for c in GL_CASES:
        try:
            print(f"measured GAMMALN from {c}: {V.load_measured_gl(c)} cells")
        except FileNotFoundError:
            pass
    data = {n: load(n) for n in names}
    keys = {n: integrity(n, rows) for n, rows in data.items() if rows}
    ns = list(keys)
    for i, p in enumerate(ns):
        for q in ns[i + 1 :]:
            sh_p = {(a, b) for _, a, b in keys[p]}
            sh_q = {(a, b) for _, a, b in keys[q]}
            print(f"overlap {p}/{q}: cells {len(keys[p] & keys[q])}, shapes {len(sh_p & sh_q)}")
    for n, rows in data.items():
        if not rows:
            continue
        rows = [r for r in rows if r["echo_ok"] is not False]
        report(n, rows, "M2 TOMS-d, Excel GL", lambda a, b: V.lnb_m2(a, b, True, "toms"))
        report(n, rows, "  same, x87 stored ops", lambda a, b: V.lnb_m2(a, b, True, "toms", "x"), "x")
        report(n, rows, "M2 TOMS-d, engine GL", lambda a, b: V.lnb_m2(a, b, False, "toms"))
        report(n, rows, "M2 accurate algdiv", lambda a, b: V.lnb_m2(a, b, True, "exact"))


if __name__ == "__main__":
    main(sys.argv[1:] or list(DATASETS))
