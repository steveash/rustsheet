"""Local step reader at isolated single-stage binade crossings (T-1941 W5 part 4; review plan 9.3 I1).
For each crossing x* of S_k or P_k (a sum stage's configuration change) with no other event within W:
  target step  = [mean rel bias right - left] of (target - c5 sim)          (relative ulps, all points)
  trial step   = [mean rel bias right - left] of (trial sim - c5 sim), trial = c5 with c_k + j*ulp(c_k)
and list the j whose trial step matches the target step (|diff| < 2 SE), over j in [-J, J].
usage: stepreader.py control|real [W]"""

import numpy as np, sys, os

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
from sc import candidate, load, res
from events import events

mode = sys.argv[1]
W = float(sys.argv[2]) if len(sys.argv) > 2 else 0.002
x, b, c = load()
m = (x >= 0.7) & (x < 1.47) & (np.abs(x - 1) > 2e-3)
x = x[m]
b = b[m]
N5 = candidate("N3")
D5 = candidate("D3")
if os.environ.get("BASE"):
    _l = open(os.environ["BASE"]).read().split("\n")
    N5 = np.array([float.fromhex(v) for v in _l[0].split()]); D5 = np.array([float.fromhex(v) for v in _l[1].split()])
XL = x.astype(np.longdouble)
ref = (
    (XL - 1) * np.polyval(N5.astype(np.longdouble)[::-1], XL) / np.polyval(D5.astype(np.longdouble)[::-1], XL)
)
U = np.abs(ref) * np.longdouble(2.0**-52)
TRUTH = {("N", 0): 37, ("D", 1): -11, ("D", 2): 3}
if mode == "control":
    Nt = N5.copy()
    Dt = D5.copy()
    for (ch, k), j in TRUTH.items():
        C = Nt if ch == "N" else Dt
        C[k] += j * np.spacing(abs(C[k]))
    tgt = res(x, Nt, Dt)
else:
    tgt = b
ev = events(N5, D5, 0.7, 1.47)
JMAX = int(os.environ.get("JMAX", 4096))
out = []
# cluster events of the same stage that lie within W of each other; isolation is w.r.t. other stages only
ev = [e for e in ev if e[1] not in ("A", "res")] + [e for e in ev if e[1] in ("A", "res")]
stagename = lambda nm: nm[0] + nm.split(".")[1][1:] if "." in nm else nm
clusters = []
for xe, nm, e0, e1 in sorted(ev):
    sn = stagename(nm)
    if clusters and clusters[-1][2] == sn and xe - clusters[-1][1] < W: clusters[-1][1] = xe; clusters[-1][3].append(nm); continue
    clusters.append([xe, xe, sn, [nm]])
sets = {}
for i, (xa, xb, sn, nms) in enumerate(clusters):
    if sn in ("A", "res") or sn == "D0": continue
    others = [cl for k, cl in enumerate(clusters) if k != i and (abs(cl[0] - xb) < W or abs(cl[1] - xa) < W or (cl[0] >= xa and cl[0] <= xb))]
    if others: continue
    L_ = (x >= xa - W) & (x < xa); R_ = (x > xb) & (x <= xb + W)
    if L_.sum() < 100 or R_.sum() < 100: continue
    ix = L_ | R_; xx = x[ix]; ll = L_[ix]; rr = R_[ix]; uu = U[ix]
    base = res(xx, N5, D5)
    def stepse(v):
        e = ((v - base) / uu).astype(float)
        return e[rr].mean() - e[ll].mean(), np.sqrt(e[rr].var() / rr.sum() + e[ll].var() / ll.sum())
    ts, tse = stepse(tgt[ix]); tse = max(tse, 0.01)
    ch = sn[0]; k = int(sn[1:]); C0 = N5 if ch == "N" else D5
    uc = np.spacing(abs(C0[k])); js = np.arange(-JMAX, JMAX + 1); steps = np.empty(len(js))
    for n, j in enumerate(js):
        N = N5.copy(); D = D5.copy(); C = N if ch == "N" else D; C[k] = C0[k] + j * uc
        steps[n] = stepse(res(xx, N, D))[0]
    ok = set(js[np.abs(steps - ts) < 2.5 * tse].tolist())
    sets.setdefault(sn, []).append((xa, ok, ts, tse, len(js)))
    tr = TRUTH.get((ch, k), 0) if mode == "control" else None
    print("%.4f %-12s target step %+.3f +- %.3f | %4d/%d consistent | c5 (j=0) %s%s" % (xa, "+".join(nms), ts, tse, len(ok), len(js), "ok" if 0 in ok else "EXCLUDED", (" | truth j=%d %s" % (tr, "ok" if tr in ok else "MISSED")) if tr is not None else ""), flush=True)
print("\nper-stage intersection of consistent moves (j in ulps of the constant):")
for sn, lst in sets.items():
    inter = set.intersection(*[o for _, o, _, _, _ in lst])
    js_ = sorted(inter)
    print("  %s: %d crossings -> %d consistent j%s%s" % (sn, len(lst), len(js_), (" e.g. " + str(js_[:10])) if js_ else "", ("  (c5 j=0 %s)" % ("in" if 0 in inter else "OUT"))))
print("\nnearest consistent moves per stage:")
best = {}
for sn, lst in sets.items():
    inter = sorted(set.intersection(*[o for _, o, _, _, _ in lst]))
    if not inter or 0 in inter: continue
    near = sorted(inter, key=abs)[:8]; best[sn] = near
    print("  %s nearest: %s" % (sn, near))
import json; json.dump({k: v for k, v in best.items()}, open(os.path.join(HERE, "nearest_%s.json" % mode), "w"))
