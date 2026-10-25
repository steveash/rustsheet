"""Per function group x bratio branch: engine visibly off, fixed / broken by a model, bit-exact cells lost; and the
gated-suite cells a port would regress (baseline F lines). Reads impact.pkl (run impact.py first).

Usage: branches.py [MODEL]   (default BEST+log1p)
"""

import sys
from collections import defaultdict

import impact
import implib as lib


def group(fn: str) -> str:
    f = lib.FAMILY.get(fn, fn)
    return {"beta": "BETA", "f": "F", "t": "T", "binom": "BINOM", "negbinom": "NEGB"}.get(f, f)


def main() -> None:
    model = sys.argv[1] if len(sys.argv) > 1 else "BEST+log1p"
    cells, models = lib.cache("impact", impact.build)
    for pop in ("pre-T2464", "w2f-probes"):
        rows = [c for c in cells if c["pop"] == pop]
        g = defaultdict(list)
        for c in rows:
            g[(group(c["fn"]), impact.branch_of(c, models))].append(c)
        print(f"\n== {pop}, model {model}: group x branch")
        print(
            f"  {'group':6s} {'branch':24s}{'n':>7s}{'eng_vis':>9s}{'fixed':>7s}{'broke':>7s}{'left':>7s}"
            f"{'eng_ex':>8s}{'mod_ex':>8s}{'lost_ex':>8s}"
        )
        for (gr, br), rs in sorted(g.items()):
            m = [models[model][c["k"]][0] for c in rs]
            ev = [not lib.within15(c["excel"], c["engine"]) for c in rs]
            mv = [lib.within15(c["excel"], v) for c, v in zip(rs, m)]
            fixed = sum(e and v for e, v in zip(ev, mv))
            broke = sum((not e) and (not v) for e, v in zip(ev, mv))
            lost = sum(c["engine"] == c["excel"] and v != c["excel"] for c, v in zip(rs, m))
            print(
                f"  {gr:6s} {br:24s}{len(rs):7d}{sum(ev):9d}{fixed:7d}{broke:7d}{sum(ev) - fixed:7d}"
                f"{sum(c['engine'] == c['excel'] for c in rs):8d}{sum(c['excel'] == v for c, v in zip(rs, m)):8d}"
                f"{lost:8d}"
            )
    print(
        f"\n== gated-suite cells whose bit-exact (baseline) or visible status the model ({model}) would regress"
    )
    for c in cells:
        if not c["gated"]:
            continue
        v, br = models[model][c["k"]]
        lost = c["engine"] == c["excel"] and v != c["excel"]
        broke = lib.within15(c["excel"], c["engine"]) and not lib.within15(c["excel"], v)
        if lost or broke:
            print(
                f"  {'LOST-EXACT' if lost else ''}{' VISIBLE' if broke else ''}  {c['fn']} {c['case']} {c['k'][1]} "
                f"{c['prob'][:6]} branch {br} excel {c['excel']!r} engine {c['engine']!r} model {v!r} "
                f"({lib.ulps(v, c['excel']):.0f} ulp)"
            )


if __name__ == "__main__":
    main()
