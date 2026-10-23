"""The beta density for min(a, b) > 1: the current engine (powf x Gamma ratio) against the correctly rounded
stand-in in BEST and r2-density-large's Loader model LDe (model.py, stir=exact, lf=LF0), on the standing pdf cells.
Reads impact.pkl (impact.py). GAMMALN prefetches of r2-density-large are redirected to this directory's cache.
"""

import sys
from collections import defaultdict

import impact
import implib as lib

sys.path.append(str(lib.T2464 / "r2-density-large"))
import dlib  # noqa: E402

dlib._GC = lib.HERE / "gl_cache_dl.pkl"
import model as LDM  # noqa: E402

OPTS = dict(stir="exact", lf="LF0")


def main() -> None:
    cells, models = lib.cache("impact", impact.build)
    rows = [c for c in cells if c["prob"][0] == "pdf" and min(c["prob"][2], c["prob"][3]) > 1]
    g = defaultdict(list)
    for c in rows:
        x, a, b, width = c["prob"][1:]
        v = float(LDM.density(x, a, b, OPTS))
        v = v / width if width != 1.0 else v
        key = (c["pop"], "max>=8" if max(a, b) >= 8 else "max<8")
        g[key].append((c, v))
    print(
        f"{'pop':12s}{'shapes':8s}{'n':>7s}{'eng_vis':>9s} | {'CR fixed/broke/lost':>20s} | "
        f"{'LDe fixed/broke/lost':>21s} | {'ex: eng/CR/LDe':>18s} | {'w15: eng/CR/LDe':>20s}"
    )
    for key, rs in sorted(g.items()):

        def stats(val):
            fixed = broke = lost = ex = w = 0
            for c, v in rs:
                m = val(c, v)
                e, eng = c["excel"], c["engine"]
                ev, mv = lib.within15(e, eng), lib.within15(e, m)
                fixed += (not ev) and mv
                broke += ev and not mv
                lost += eng == e and m != e
                ex += m == e
                w += mv
            return fixed, broke, lost, ex, w

        cr = stats(lambda c, v: models["BEST"][c["k"]][0])
        ld = stats(lambda c, v: v)
        en = stats(lambda c, v: c["engine"])
        vis = sum(not lib.within15(c["excel"], c["engine"]) for c, _ in rs)
        print(
            f"{key[0]:12s}{key[1]:8s}{len(rs):7d}{vis:9d} | {cr[0]:6d}/{cr[1]:5d}/{cr[2]:6d}    | "
            f"{ld[0]:6d}/{ld[1]:5d}/{ld[2]:6d}     | {en[3]:5d}/{cr[3]:5d}/{ld[3]:5d}  | "
            f"{en[4]:6d}/{cr[4]:6d}/{ld[4]:6d}"
        )


if __name__ == "__main__":
    main()
