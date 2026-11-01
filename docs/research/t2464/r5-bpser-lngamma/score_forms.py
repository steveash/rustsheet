"""Score the two lnGamma(a0 < 1) forms of bpser's L on probe t3570-r5-bpser-lngamma-beta (make_probe.py).

GL53: L = fl(GL(a0) + fl(GL(b0) - GL(a0 + b0))), every GL Excel's measured GAMMALN (probe -gl).
L53:  the same with GL(a0) replaced by TOMS's fl(gamln1(a0) - LN(a0)).
Prints exact / within 4 ulp / within 15 digits per run (b0, center, x) and per side of the center, and, on the cells
where the two forms give different values, which one Excel matches. Tune = b0 2.5, hold-out = b0 6.25 (fixed before
scoring).
usage: score_forms.py
"""

from __future__ import annotations

import json
from collections import defaultdict

import make_probe as mp

lib = mp.lib
CASES = lib.CASES


def sidecar(name: str) -> dict:
    return json.load(open(CASES / f"{name}.oracle.json"))["sheets"]["Calc"]


def measured_gl() -> dict:
    out = sidecar("t3570-r5-bpser-lngamma-gl")
    got = {}
    for i, _, v, _, _ in json.load(open(mp.HERE / "t3570-r5-bpser-lngamma-gl.meta.json")):
        c = out.get(f"A{i}")
        if c and c.get("t") == "n":
            got[v] = float(c["v"])
    return got


def main() -> None:
    gl = measured_gl()
    lib.bx._GL.update(gl)
    out = sidecar("t3570-r5-bpser-lngamma-beta")
    rows = []
    for i, blk, x, a, b in json.load(open(mp.HERE / "t3570-r5-bpser-lngamma-beta.meta.json")):
        c = out.get(f"A{i}")
        if c and c.get("t") == "n":
            rows.append((blk, x, a, b, float(c["v"])))
    forms = {"GL53": lib.V(lg="GL53"), "L53": lib.V(lg="L53")}
    vals = {nm: [lib.kernel(a, b, x, 1.0 - x, v)[0] for _, x, a, b, _ in rows] for nm, v in forms.items()}
    assert not lib.bx._GL_MISSING, "a GAMMALN argument is not in the -gl probe"
    print(f"{len(rows)} cells, {len(gl)} measured GAMMALN")
    groups = defaultdict(list)
    for k, (blk, _, a, _, _) in enumerate(rows):
        b0, c, _ = blk.split(":")
        side = "below" if a < float(c) else "above"
        groups[(blk, "")].append(k)
        groups[(f"{b0}:{c}", side)].append(k)
        groups[("b0=" + b0, "tune" if b0 == "2.5" else "hold")].append(k)
    print(f"{'group':28} {'n':>4}  " + "  ".join(f"{nm:>14}" for nm in forms) + "  differ: GL53/L53/neither")
    for key in sorted(groups):
        ks = groups[key]
        cols = []
        for nm in forms:
            n, ex, f4, w = lib.tally([(vals[nm][k], rows[k][4]) for k in ks])
            cols.append(f"{ex}/{f4}/{w}")
        d = [k for k in ks if vals["GL53"][k] != vals["L53"][k]]
        g = sum(vals["GL53"][k] == rows[k][4] for k in d)
        t = sum(vals["L53"][k] == rows[k][4] for k in d)
        print(f"{key[0] + ' ' + key[1]:28} {len(ks):>4}  " + "  ".join(f"{c:>14}" for c in cols) + f"  {len(d)}: {g}/{t}/{len(d) - g - t}")


if __name__ == "__main__":
    main()
