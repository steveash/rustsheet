"""Needed exponent: each interior BINOM pmf cell's admissible E set (EXP_V6(E) == Excel) against the exact ln pmf.
Prints, per (n, p), the share of cells with a nonempty E set and the offset (E_mid - exact) in units of 2^-53."""
import collections
import math
import pickle
from pathlib import Path

import mpmath
import blib as B
import r3lib as R3

CP = Path(__file__).with_name("eset.pkl")
if CP.exists():
    R3._ES.update(pickle.loads(CP.read_bytes()))
rows = [r for r in B.load_cells() if r["fn"] == "BIN" and 0 < r["k"] < r["n"]]
out = collections.defaultdict(list)
for r in rows:
    es = R3.eset(r["excel"])
    lx = mpmath.log(B.exact_pmf("BIN", r["k"], r["n"], r["p"]))
    if es:
        mid = (es[0] + es[-1]) / 2
        out[(r["n"], r["p"])].append(float((mid - lx) * 2**53))
    else:
        out[(r["n"], r["p"])].append(None)
CP.write_bytes(pickle.dumps(R3._ES))
for key in sorted(out):
    v = out[key]
    ok = [x for x in v if x is not None]
    if not ok:
        print(key, len(v), "no E")
        continue
    m = sum(ok) / len(ok)
    sd = math.sqrt(sum((x - m) ** 2 for x in ok) / len(ok))
    print(key, f"n={len(v)} haveE={len(ok)} mean={m:+.2f} sd={sd:.2f} min={min(ok):+.1f} max={max(ok):+.1f}")
