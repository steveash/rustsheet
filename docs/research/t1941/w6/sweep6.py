# T-1941 W6: per-run ceiling of the op structure (reviewer 3's sweep.py idea on the W6 kernel). Within one run of
# consecutive doubles any coefficient change acts on each stage as a CONSTANT pre-rounding offset, so the best exact
# fraction over offsets at the most sensitive stages bounds what ANY coefficients of this structure can reach there.
# Per run: offsets c1 at N.S1 and g1 at D.S1 on grids (units of each stage's ulp), and the last sums N.S0 / D.S0 by
# exact interval coverage (point i is reproduced iff fl(S0/Dfin) is one of the data's R candidates, a union of unit
# intervals in the S0 offset); N and D alternate for a few rounds. Usage: sweep6.py RFILE CHAIN [nruns] [R1] [STEP1]
import sys, re, collections, numpy as np

sys.path.insert(0, sys.path[0])
from lib6 import L, st
from synth import read_chain, nrm

rf, chain = sys.argv[1], sys.argv[2]
nruns = int(sys.argv[3]) if len(sys.argv) > 3 else 4
R1 = float(sys.argv[4]) if len(sys.argv) > 4 else 32.0
S1 = float(sys.argv[5]) if len(sys.argv) > 5 else 0.125
N, D = read_chain(chain)
N = np.array(N) /nrm(D)
D = np.array(D) /nrm(D)
pts = collections.defaultdict(list)
for l in open(rf):
    q = l.split()
    pts[re.sub(r"[a-d]?(:B)?$", "", q[5])].append(
        (float.fromhex(q[0]), float.fromhex(q[2]), float.fromhex(q[3]))
    )
runs = []
for v in pts.values():
    v = np.array(sorted(v))
    br = np.where(np.nextafter(v[:-1, 0], np.inf) != v[1:, 0])[0] + 1
    runs += [v[s] for s in np.split(np.arange(len(v)), br) if len(s) >= 4096]
runs.sort(key=lambda r: r[0, 0])
pick = [runs[i] for i in np.linspace(0, len(runs) - 1, nruns).astype(int)]
ms = np.arange(-3, 4)


def stages(C, x, off1):
    """stored Horner down to S1 (with constant offset off1 ulps at S1 pre-rounding); returns stored S1 and S0 pre-value
    (without its offset) and ulp(S0)."""
    s = L(C[-1])
    for j in range(len(C) - 2, 0, -1):
        v = L(st(s * L(x))) + L(C[j])
        if j == 1:
            v = v + L(off1) * L(np.spacing(np.abs(st(v))))
        s = L(st(v))
    v0 = L(st(s * L(x))) + L(C[0])
    return v0, np.spacing(np.abs(st(v0))).astype(L)


def cover(v0, u0, other, lo, hi, num):
    """best offset c (ulps of S0) of this chain's last sum: point reproduced iff fl(S0/other) (num) or fl(other/S0)
    lies in [lo, hi] for S0 = RN(v0 + c u0); returns (count, c)."""
    s0b = st(v0)
    base = np.array((L(s0b) - v0) / u0, dtype=float)
    ev, sg = [], []
    for m in ms:
        s0 = st(L(s0b) + L(m) * u0)
        R = st(L(s0) / L(other)) if num else st(L(other) / L(s0))
        ok = (R >= lo) & (R <= hi)
        a = base[ok] + m - 0.5
        ev += [a, a + 1.0]
        sg += [np.ones(len(a)), -np.ones(len(a))]
    ev = np.concatenate(ev)
    sg = np.concatenate(sg)
    if len(ev) == 0:
        return 0, 0.0
    o = np.lexsort((sg, ev))
    cs = np.cumsum(sg[o])
    k = int(np.argmax(cs))
    return int(cs[k]), float(ev[o][k])


for r in pick:
    x, lo, hi = r[:, 0], r[:, 1], r[:, 2]
    n = len(x)
    vN, uN = stages(N, x, 0.0)
    vD, uD = stages(D, x, 0.0)
    Nf, Df = st(vN), st(vD)
    base = np.mean((st(L(Nf) / L(Df)) >= lo) & (st(L(Nf) / L(Df)) <= hi))
    best = (base, 0.0, 0.0, 0.0, 0.0)
    cN1 = cD1 = 0.0
    for rnd in range(3):
        bN = (-1, 0, 0)
        for c1 in np.arange(-R1, R1 + 1e-9, S1):
            v, u = stages(N, x, c1)
            cnt, c0 = cover(v, u, Df, lo, hi, True)
            if cnt > bN[0]:
                bN = (cnt, c1, c0)
        cN1 = bN[1]
        v, u = stages(N, x, cN1)
        Nf = st(v + L(bN[2] + 1 / 64) * u)
        bD = (-1, 0, 0)
        for g1 in np.arange(-R1, R1 + 1e-9, S1):
            v, u = stages(D, x, g1)
            cnt, g0 = cover(v, u, Nf, lo, hi, False)
            if cnt > bD[0]:
                bD = (cnt, g1, g0)
        cD1 = bD[1]
        v, u = stages(D, x, cD1)
        Df = st(v + L(bD[2] + 1 / 64) * u)
        frac = np.mean((st(L(Nf) / L(Df)) >= lo) & (st(L(Nf) / L(Df)) <= hi))
        print(
            "  x0 %.4f round %d: N.S1 %+.3f N.S0 %+.3f -> %.4f | D.S1 %+.3f D.S0 %+.3f -> %.4f (simulated %.4f)"
            % (x[0], rnd, bN[1], bN[2], bN[0] / n, bD[1], bD[2], bD[0] / n, frac),
            flush=True,
        )
    print("run x0 %.4f n %d: chain exact %.4f -> per-run ceiling %.4f" % (x[0], n, base, frac), flush=True)
