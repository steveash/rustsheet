"""Top-down peeling, one coefficient at a time (T-1941 W5 part 5).
With every coefficient above c_j already exact, Delta S_j(x) = Delta c_j (a constant), so
  sum line S_j:          phi = Delta c_j / u_S          (mod 1)
  product line P_{j-1}:  phi = Delta c_j * x / u_P      (mod 1; x-dependence fixes the full integer)
Objective O(k) = sum_r C_S,r(k ulp/u_S) + C_P,r(k ulp x_r/u_P) over integer moves k of c_j, |k| <= KMAX.
Reports the top k with Z against the scan's own distribution; with APPLY=1 writes the moved set, refitting the lower
coefficients (below j, except the read ones) to keep the function (fcorrect-style least squares, no pins).
usage: peel.py control|real CHAIN J [OUT]   env: BASE, TRUTHFILE (control), KMAX, SEED, TSIZE"""

import numpy as np, sys, os, time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
from sc import res
from meter import sites_and_weights, template, resid, L

mode, CH, J = sys.argv[1], sys.argv[2], int(sys.argv[3])
OUT = sys.argv[4] if len(sys.argv) > 4 else None


def ld(f):
    l = open(f).read().split("\n")
    return np.array([float.fromhex(v) for v in l[0].split()]), np.array(
        [float.fromhex(v) for v in l[1].split()]
    )


N0, D0 = ld(os.environ.get("BASE", os.path.join(HERE, "..", "p4", "ss_fc.txt")))
ROWS = os.environ.get(
    "ROWS", "/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows2.txt"
)
z = np.load(ROWS + ".near1.npz")
x = z["x"]
b = z["b"]
LO, HI = float(os.environ.get("LO", 0.7)), float(os.environ.get("HI", 1.47))
RL = int(os.environ.get("RL", 4096))
br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0] + 1
segs = [
    s[:RL]
    for s in np.split(np.arange(len(x)), br)
    if len(s) >= 256 and LO <= x[s[0]] < HI and abs(x[s[0]] - 1) > 2e-3
]
if mode == "control":
    Nt, Dt = ld(os.environ["TRUTHFILE"])
kS = (CH, "S", J)
kP = (CH, "P", J - 1)
C0 = N0 if CH == "N" else D0
uc = np.spacing(abs(C0[J]))


def curves(key, W, r):
    pre, sv, w = W[key]
    T = template(pre, sv, w)
    Tc = T - T.mean(1, keepdims=True)
    nrm = np.sqrt((Tc * Tc).sum(1))
    return (Tc @ r) / np.maximum(nrm, 1e-30) / max(np.sqrt(r @ r), 1e-30) * np.sqrt(len(r)), float(
        np.median(np.spacing(np.abs(sv)))
    )


CS = []
CP = []
xm = []
uS = []
uP = []
for s in segs:
    xx = x[s]
    W, ref = sites_and_weights(N0, D0, xx)
    r = resid(xx, res(xx, Nt, Dt) if mode == "control" else b[s], ref)
    c1, u1 = curves(kS, W, r)
    c2, u2 = curves(kP, W, r)
    CS.append(c1)
    CP.append(c2)
    uS.append(u1)
    uP.append(u2)
    xm.append(xx[len(xx) // 2])
CS = np.array(CS)
CP = np.array(CP)
xm = np.array(xm)
uS = np.array(uS)
uP = np.array(uP)
R = len(xm)


def obj(C, ph):
    ph = ph % 1.0
    k = np.floor(ph * 64).astype(int) % 64
    fr = ph * 64 - np.floor(ph * 64)
    idx = np.arange(C.shape[0])[None, :]
    return (C[idx, k] * (1 - fr) + C[idx, (k + 1) % 64] * fr).sum(-1)


KMAX = int(float(os.environ.get("KMAX", 2e5)))
ks = np.arange(-KMAX, KMAX + 1)
vals = []
for i0 in range(0, len(ks), 4000):
    kk = ks[i0 : i0 + 4000][:, None] * uc
    vals.append(obj(CS, kk / uS[None, :]) + obj(CP, kk * xm[None, :] / uP[None, :]))
vals = np.concatenate(vals)
mu, sd = np.median(vals), np.std(vals)
o = np.argsort(vals)[::-1]
print(
    "%s.%d: runs %d, k range +-%d; top: %s"
    % (
        CH,
        J,
        R,
        KMAX,
        " | ".join("k=%+d %.1f (Z %.1f)" % (ks[i], vals[i], (vals[i] - mu) / sd) for i in o[:5]),
    )
)
if mode == "control":
    Ct = Nt if CH == "N" else Dt
    kt = int(round((Ct[J] - C0[J]) / uc))
    i = np.where(ks == kt)[0]
    print(
        "   truth k=%+d value %s"
        % (
            kt,
            ("%.1f (Z %.1f, rank %d)" % (vals[i[0]], (vals[i[0]] - mu) / sd, int(np.where(o == i[0])[0][0])))
            if len(i)
            else "out of range",
        )
    )
if OUT and os.environ.get("APPLY"):
    k = int(ks[o[0]])
    N1 = N0.copy()
    D1 = D0.copy()
    C1 = N1 if CH == "N" else D1
    C1[J] += k * uc
    # keep the function: least squares on the coefficients below J of the same chain plus all of the other chain's
    # coefficients below its top visible part, excluding the plateau-read ones
    READ = {("N", 0), ("N", 1), ("N", 2), ("D", 1), ("D", 2), ("D", 3)}
    free = [(CH, i) for i in range(J) if (CH, i) not in READ] + [
        (o2, i) for o2 in ("N", "D") if o2 != CH for i in range(1, J) if (o2, i) not in READ
    ]
    xs = np.linspace(0.7, 1.5, 120).astype(L)
    tgt = np.polyval(N0.astype(L)[::-1], xs) / np.polyval(D0.astype(L)[::-1], xs)
    for it in range(3):
        Nv = np.polyval(N1.astype(L)[::-1], xs)
        Dv = np.polyval(D1.astype(L)[::-1], xs)
        M = np.array(
            [
                (np.spacing(abs((N1 if c == "N" else D1)[i])) * xs**i / (Nv if c == "N" else -Dv)).astype(
                    float
                )
                for c, i in free
            ]
        ).T
        sol, *_ = np.linalg.lstsq(M / 1e-18, ((tgt * Dv / Nv) - 1).astype(float) / 1e-18, rcond=None)
        for (c, i), v in zip(free, sol):
            CC = N1 if c == "N" else D1
            CC[i] += round(v) * np.spacing(abs(CC[i]))
    f = np.abs((np.polyval(N1.astype(L)[::-1], xs) / np.polyval(D1.astype(L)[::-1], xs)) / tgt - 1).max()
    open(OUT, "w").write(" ".join(v.hex() for v in N1) + "\n" + " ".join(v.hex() for v in D1) + "\n")
    print("   applied k=%+d to %s%d; function restored to %.1e; wrote %s" % (k, CH, J, float(f), OUT))
if os.environ.get('SPLIT') and mode == 'control':
    for kk_ in [kt, -1, 1, 0]:
        a_ = obj(CS, np.array([[kk_*uc]])/uS[None, :])[0]; b_ = obj(CP, np.array([[kk_*uc]])*xm[None, :]/uP[None, :])[0]
        print('   k=%+d: S-site %.1f  P-site %.1f' % (kk_, a_, b_))
    print('   u_S values', np.unique(uS)[:5], 'u_P values', np.unique(uP)[:5], 'ulp(c_J)', uc)
