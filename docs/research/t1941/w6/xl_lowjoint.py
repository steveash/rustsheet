# T-1941 W6: exact joint fit (mpjoint.joint_solve) of Excel's 6 held tracks plus low-stage tracks found by lowfit.py,
# from a pinned-upper-stage chain; relinearises ITERS times and reports per-track rms. Usage:
#   xl_lowjoint.py CHAIN TRACKS OUT "ti:ch:law:j:sign,..." [ITERS]   (law T or Pn1/Pn2/..., e.g. 27:D:Pn1:2:+1)
import sys, json, numpy as np
sys.path.insert(0, sys.path[0])
from synth import read_chain, nrm
from mpjoint import joint_solve
from lawrows import predict
from fastenum import prep
N, D = read_chain(sys.argv[1]); N = np.array(N) /nrm(D); D = np.array(D) /nrm(D)
T = json.load(open(sys.argv[2]))
spec = [a.split(":") for a in sys.argv[4].split(",")]
it = int(sys.argv[5]) if len(sys.argv) > 5 else 4
assign = []
for ti, ch, law, j, sg in spec:
    ti, j, sg = int(ti), int(j), int(sg)
    x, F = prep(T[ti], 12)
    kk = int(np.round(np.median(predict(N, D, ch, law, j, x) - sg * F)))
    assign.append((ti, (ch, law, j, sg, kk, 0)))
for k in range(it):
    N, D, res = joint_solve(T, assign, N, D, len(N) - 1, len(D) - 1)
    print("iter %d: worst %.1e  " % (k, max(res)) + " ".join("%d:%s.%s%d %.1e" % (ti, h[0], h[1], h[2], r) for (ti, h), r in zip(assign, res)), flush=True)
open(sys.argv[3], "w").write("N " + " ".join(float(v).hex() for v in N) + "\nD " + " ".join(float(v).hex() for v in D) + "\n")
