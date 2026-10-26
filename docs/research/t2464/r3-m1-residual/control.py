"""Positive controls. (1) Plant: synthesise 'Excel' values from M2 with lnB moved by a known number of ulps per shape
(probe a's x and shapes) and check that the per-shape scan (lscan, mode m2) recovers every planted shift at 100%.
(2) Discrimination: M2's score on synthetic values from M1 (and M1's on M2's), so a hit rate near 100% is not
automatic. (3) Echo check: rows of probes a / hold whose read-back arguments differ from the intended doubles.
Usage: control.py"""
import json
import random

import m1lib as M
from lscan import scan

if __name__ == "__main__":
    rows = M.load_own("a")
    rng = random.Random(5)
    shapes = sorted({(r["a"], r["b"]) for r in rows})
    plant = {sh: rng.randint(-4, 4) for sh in shapes}
    syn = []
    for r in rows:
        sh = (r["a"], r["b"])
        tx, ty = M.pieces(r["x"], *sh)
        v = M.combine(tx, ty, M.nb(M.lnb_base(*sh), plant[sh]), sh[0], sh[1], "m2")
        syn.append(dict(r, excel=v))
    res = scan(syn, "m2", 6)
    ok = sum(max(h, key=lambda j: h[j]) == plant[sh] and h[plant[sh]] == n for sh, (n, h, _) in res.items())
    print(f"(1) planted lnB shifts recovered at 100%: {ok} of {len(res)} shapes")
    s1 = [dict(r, excel=M.combine(*M.pieces(r["x"], r["a"], r["b"]), M.lnb_base(r["a"], r["b"]), r["a"], r["b"], "m1"))
          for r in rows]
    hit = sum(M.combine(*M.pieces(r["x"], r["a"], r["b"]), M.lnb_base(r["a"], r["b"]), r["a"], r["b"], "m2")
              == r["excel"] for r in s1)
    print(f"(2) M2 on synthetic M1 values: {hit} of {len(s1)} bit-exact")
    for nm in ("a", "hold"):
        meta = json.loads((M.HERE / f"probe_{nm}.meta.json").read_text())
        print(f"(3) probe {nm}: {len(meta['rows'])} rows, {len(M.load_own(nm))} with exact echoes and a number")
