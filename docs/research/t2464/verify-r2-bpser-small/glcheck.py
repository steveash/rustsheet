"""Engine GAMMALN model vs Excel's measured GAMMALN (probe t2464-r2-bpser-small-gl), by argument bin."""
import sys
from collections import Counter
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify as v
v.load_gl_probe()
xs = sorted(v._GL)
eng = v.em.excel("GAMMALN", xs)
c = Counter()
for x, e in zip(xs, eng):
    b = "<1" if x < 1 else "[1,2)" if x < 2 else "[2,8)" if x < 8 else ">=8"
    c[(b, "n")] += 1
    c[(b, "exact")] += e == v._GL[x]
print(sorted(c.items()))
