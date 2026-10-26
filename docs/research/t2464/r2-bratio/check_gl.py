"""Engine GAMMALN model vs Excel's GAMMALN (probe t2464-r2-bratio-gammaln) at the kernel's arguments."""
import json
import math
import pickle
from collections import Counter

from bratio_excel import HERE

args = json.load(open(HERE / "t2464-r2-bratio-gammaln.args.json"))
out = json.load(open(HERE / "t2464-r2-bratio-gammaln.out.json"))["sheets"]["Calc"]
model = pickle.loads((HERE / "gammaln_cache.pkl").read_bytes())
excel = {}
by = Counter()
miss = []
for i, v in enumerate(args):
    c = out.get(f"A{i + 1}")
    if not c or c.get("t") != "n":
        continue
    excel[v] = c["v"]
    m = model[v]
    d = 0 if m == c["v"] else round((m - c["v"]) / math.ulp(c["v"]))
    band = "v<1" if v < 1 else "1<=v<2" if v < 2 else "2<=v<8" if v < 8 else "v>=8"
    by[(band, d == 0)] += 1
    if d:
        miss.append((v, d))
print("cells", len(excel))
for k in sorted(by):
    print(k, by[k])
print("misses (arg, model - excel in ulps):", miss[:40])
