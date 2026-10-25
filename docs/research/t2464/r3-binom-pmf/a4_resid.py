"""Where the Loader model (r3 knobs, exact stirlerr, extended chain) leaves residuals: D - dX by |E| and k/n."""
import collections
import math
import sys

import numpy as np

import blib as B

LDB = np.longdouble
st = B.st
o = eval(sys.argv[1]) if len(sys.argv) > 1 else dict(stir="ex", chain="e")
rows = [r for r in B.load_cells() if r["fn"] == "BIN" and 0 < r["k"] < r["n"] and 1e-5 < r["p"] and r["n"] <= 1000]
g = collections.defaultdict(list)
for r in rows:
    E = B.loader_E(r["k"], r["n"], r["p"], st(LDB(1) - LDB(r["p"])), o)
    res = B.D_units(r) - B.units(E, r)
    aE = abs(float(E))
    band = "E<1" if aE < 1 else "E<4" if aE < 4 else "E<16" if aE < 16 else "E<64" if aE < 64 else "E>=64"
    g[band].append(res)
    r["res"] = res
    r["E"] = float(E)
for band in ("E<1", "E<4", "E<16", "E<64", "E>=64"):
    v = g[band]
    if v:
        print(band, len(v), "rms %.2f" % math.sqrt(sum(x * x for x in v) / len(v)), "median|.| %.2f" % sorted(map(abs, v))[len(v) // 2])
print("worst")
for r in sorted(rows, key=lambda r: -abs(r["res"]))[:25]:
    print(r["k"], r["n"], r["p"], "E=%.3f" % r["E"], "res=%.1f" % r["res"], r["case"].split("/")[-1], r["cell"])
print("small |E| <1 sample")
for r in [r for r in rows if abs(r["E"]) < 4][:40]:
    print(r["k"], r["n"], r["p"], "E=%.3f" % r["E"], "res=%.2f" % r["res"])
