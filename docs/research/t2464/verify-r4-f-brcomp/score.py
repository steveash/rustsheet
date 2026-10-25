"""Re-score CR and M-xd (vlib's own re-implementation) on named sets; optional --worker also runs the worker's
ratlib M-xd (imported only for a cell-for-cell agreement check).
Usage: score.py [--worker] SET ...   SET = a probes-numeric case id (without .case.json) or 'cells' / 'cells0'
('cells0' also takes cumulative flag 0 as FALSE)."""
import sys

import vlib as V

WORKER = "--worker" in sys.argv
sets = [s for s in sys.argv[1:] if not s.startswith("--")]
if WORKER:
    sys.path.insert(0, str(V.T / "r4-f-brcomp"))
    import ratlib as RL  # noqa: E402

    OPT = dict(pw="xd", hl="ext", grp="none", est="ext", L="xl")

for s in sets:
    if s in ("cells", "cells0"):
        rows, bad = V.tsv_rows(("False",) if s == "cells" else ("False", "0.0")), 0
    else:
        rows, bad = V.case_rows(s)
    cr = [float(V.f_exact(r["x"], r["d1"], r["d2"])) for r in rows]
    md = V.mxd(rows)
    print(f"# {s}: {len(rows)} rows ({bad} dropped on x echo mismatch)")
    for name, vals in (("CR", cr), ("M-xd(verifier)", md)):
        parts = [f"{name:16s} all {V.fmt(V.tally(zip(vals, (r['excel'] for r in rows))))}"]
        for c in ("max>=50", "min<=2", "other"):
            sub = [(v, r["excel"]) for v, r in zip(vals, rows) if V.cls(r) == c]
            if sub:
                parts.append(f"{c} {V.fmt(V.tally(sub))}")
        print(" | ".join(parts))
    miss = [(v, r["excel"]) for v, r in zip(md, rows) if v != r["excel"]]
    one = sum(abs(v - e) <= V.math.ulp(e) for v, e in miss)
    print(f"  M-xd misses {len(miss)}, {one} of them within 1 ulp")
    if WORKER:
        wv = [float(RL.value(r["x"], r["d1"], r["d2"], OPT)) for r in rows]
        dif = sum(a != b for a, b in zip(wv, md))
        print(f"  worker ratlib M-xd vs verifier M-xd: {dif} cells differ of {len(rows)}; worker scores "
              f"{V.fmt(V.tally(zip(wv, (r['excel'] for r in rows))))}")
