"""Per-band scores on the verifier hold-out (and hold2): LT1e, exact C, CR, and the log-branch cut alternatives
(x^2 > n claimed; R 2.x x^2 > 0.2 n; no log branch), all with the exact constant and with LT1e's.
usage: vband.py [set ...]   (default vhold hold2)"""

from __future__ import annotations

import collections
import sys

import numpy as np

import vlib as V
import vrun as R

TD = R.TD


def cut_model(r, off: dict, o: dict) -> float:
    n = r["n"]
    C = TD.ld_of(TD.loader_const_exact(n)) + np.longdouble(off.get(n, 0.0)) * TD.U53
    return TD.loader_dt(abs(r["t"]), n, C, o)


def main():
    names = sys.argv[1:] or ["vhold", "hold2"]
    rows, _ = V.load_all()
    S = R.sets(rows)
    _, _, etab = R.offsets()
    lt1e = etab()
    cuts = {
        "cut x2>n (LT1)": dict(TD.LT1),
        "cut x2>0.2n": dict(TD.LT1, bcut=0.2),
        "no log branch": dict(TD.LT1, br="none"),
    }
    for nm in names:
        rr = S[nm]
        print(
            f"== {nm}: {len(rr)} cells; t read back for {sum(r['t_readback'] is not None for r in rr)}, "
            f"mismatches {sum(r['t_readback'] is not None and r['t_readback'] != r['t'] for r in rr)}"
        )
        band = collections.defaultdict(list)
        for r in rr:
            band[round(r["t"] ** 2 / r["n"], 1)].append(r)
        for b in sorted(band):
            q = band[b]
            line = f"  q~{b:<5g} n={len(q):4d} CR {V.fmt(V.score((V.cr(r['t'], r['n']), r['excel']) for r in q))}"
            line += f" | LT1e {V.fmt(V.score((R.model(r, lt1e), r['excel']) for r in q))}"
            for cn, o in cuts.items():
                line += f" | {cn}: exC {V.score((cut_model(r, {}, o), r['excel']) for r in q)[1]}"
            print(line)
        for cn, o in cuts.items():
            print(
                f"  total {cn}: exactC {V.fmt(V.score((cut_model(r, {}, o), r['excel']) for r in rr))}; "
                f"LT1e C {V.fmt(V.score((cut_model(r, lt1e, o), r['excel']) for r in rr))}"
            )


if __name__ == "__main__":
    main()
