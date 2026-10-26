"""Re-score CR, dd and ddl (own implementation, vlib.model) on a source.  Usage: vscore.py SRC...
SRC: vhold (this directory's fresh hold-out), hold, a (r4-f-power's probes), r3hold (t2513-r3-fdensity-hold), cells."""
import collections
import sys

import vlib as V

SRCS = {
    "vhold": ["probes-numeric/t2586-verify-r4-f-power-hold.case.json"],
    "hold": ["probes-numeric/t2586-r4-f-power-hold.case.json"],
    "a": ["probes-numeric/t2586-r4-f-power-a.case.json"],
    "r3hold": ["probes-numeric/t2513-r3-fdensity-hold.case.json"],
}


def rows(src):
    cases = V.cells_tsv_cases() if src == "cells" else SRCS[src]
    out = []
    for c in cases:
        out += V.fdist_rows(c)
    return [r for r in out if r["x"] > 0 and r["d1"] >= 1 and r["d2"] >= 1]


def prebatch(rs):
    V.ln_batch([r["x"] for r in rs])
    extra = []
    for r in rs:
        mx = V.x87("*", r["d1"], r["x"])
        q = V.x87("/", mx, r["d2"]) if mx <= r["d2"] else V.x87("/", r["d2"], mx)
        if q > 0.375:
            extra.append(V.x87("+", 1.0, q))
    V.ln_batch(extra)


def main():
    vars_ = ("CR", "m1", "dd", "ddl")
    if sys.argv[1] == "--ablate":
        vars_ = ("dd", "dd-P64", "dd-Pex", "dd-S1x", "dd-nolo")
        del sys.argv[1]
    for src in sys.argv[1:]:
        rs = rows(src)
        prebatch(rs)
        for var in vars_:
            g = collections.defaultdict(list)
            for r in rs:
                m, n = r["d1"], r["d2"]
                v = V.model(r["x"], m, n, var)
                cls = "min<=2" if min(m, n) <= 2 else ("max>=50" if max(m, n) >= 50 else "other")
                side = "s1" if V.x87("*", m, r["x"]) <= n else "s2"
                for k in ("all", cls, f"{cls}/{side}"):
                    g[k].append((v, r["excel"]))
            print(f"{src:7s} {var:4s} " + " | ".join(f"{k} %d/%d/%d of %d" % V.tally(p) for k, p in sorted(g.items())))
            sys.stdout.flush()


if __name__ == "__main__":
    main()
