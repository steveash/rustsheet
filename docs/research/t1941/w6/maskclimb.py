# T-1941 W6: which low op sites does Excel store? Greedy climb over the stored/unstored flag of chosen sites (synth
# KEEP model, x87 80-bit for unstored ops), exact-R score. Usage: maskclimb.py RFILE CHAIN "N.P3,N.S3,..." [npts] [start]
import sys, numpy as np

sys.path.insert(0, sys.path[0])
from synth import read_chain, nrm, simR

rf, chain, sites = sys.argv[1], sys.argv[2], sys.argv[3].split(",")
npts = int(sys.argv[4]) if len(sys.argv) > 4 else 200000
keep = set(sys.argv[5].split(",")) - {""} if len(sys.argv) > 5 else set()
rows = [l.split() for l in open(rf)]
idx = np.random.default_rng(0).choice(len(rows), min(npts, len(rows)), replace=False)
x = np.array([float.fromhex(rows[i][0]) for i in idx])
lo = np.array([float.fromhex(rows[i][2]) for i in idx])
hi = np.array([float.fromhex(rows[i][3]) for i in idx])
del rows
N, D = read_chain(chain)
N = [v / nrm(D) for v in N]
D = [v / nrm(D) for v in D]


def score(k):
    R = simR(N, D, x, k or {"none"})
    return np.mean((R >= lo) & (R <= hi))


cur = score(keep)
print("start %.4f keep=%s" % (cur, ",".join(sorted(keep))), flush=True)
while True:
    best = (cur, None)
    for s in sites:
        v = score(keep ^ {s})
        if v > best[0]:
            best = (v, s)
    if best[1] is None:
        break
    keep ^= {best[1]}
    cur = best[0]
    print("  toggle %s -> %.4f" % (best[1], cur), flush=True)
print("final %.4f unstored=%s" % (cur, ",".join(sorted(keep))), flush=True)
