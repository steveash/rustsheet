"""FFT peel scan for one coefficient c_j over a wide integer range (T-1941 W5 part 5).
Hypothesis k (ulps of c_j). With R = max_p u_S(p)/uc (a power of two), write k = R*q + r. For each residue r, S_j is
simulated exactly (S_r = RN(P_j + c_j + r*uc)); adding q*R*uc is then exact at every point, so the S-site error is
fixed per residue and the product P_{j-1} = RN((S_r + q R uc) x) error is a sawtooth in q with per-point frequency
R*uc*x/u_P. Objective(k) = sum_p wS eS(r) r_p + sum_p wP eP(q, r) r_p (r_p = residual, zero-mean per run), the P part
via harmonic expansion and one FFT per (residue, harmonic).
usage: fscan.py control|real CHAIN J   env: BASE, TRUTHFILE, M (q range), H, TOP"""

import numpy as np, sys, os, time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from hypolib import Data, ld
from meter import L, st

mode, CH, J = sys.argv[1], sys.argv[2], int(sys.argv[3])
N0, D0 = ld(os.environ.get("BASE", os.path.join(HERE, "..", "p4", "ss_fc.txt")))
d = Data(N0, D0, mode, os.environ.get("TRUTHFILE"))
C = N0 if CH == "N" else D0
uc = float(np.spacing(abs(C[J])))
x = d.x
# stored chain down to P_J (base coefficients above J are taken as exact)
s = np.full(len(x), C[-1])
for j in range(len(C) - 2, J - 1, -1):
    p = st(L(s) * L(x))
    if j == J:
        PJ = p
        break
    s = st(L(p) + L(C[j]))
F = d.weights(CH, J, J + 1)  # F[0]: stage J-1 weight, F[1]: stage J weight (units 2^-52 per absolute)
wP, wS = F[0], F[1]
S0 = st(L(PJ) + L(C[J]))
uS = np.spacing(np.abs(S0))
R = int(max(1, 2 ** np.ceil(np.log2(max(1.0, (uS / uc).max())))))
KR = float(os.environ.get('KRANGE', 2e6))   # total k range +-KRANGE
M = int(KR / R) + 1
H = int(os.environ.get("H", 6))
NF = 1 << int(min(25, max(18, np.ceil(np.log2(M * 64)))))
qs = np.arange(-M, M + 1)
best = []
t0 = time.time()
for r in range(R):
    se = L(PJ) + L(C[J]) + L(r) * L(uc)
    Sr = st(se)
    eS = (L(Sr) - se).astype(float)
    oS = float(np.sum(wS * eS * d.r))
    v = L(Sr) * L(x)
    uP = np.spacing(np.abs(st(v))).astype(L)
    th0 = v / uP
    th0 = (th0 - np.floor(th0)).astype(float)
    fr = (L(R * uc) * L(x) / uP).astype(float)
    a = wP * d.r * uP.astype(float)
    out = np.zeros(len(qs))
    for h in range(1, H + 1):
        z = a * ((-1) ** h / (np.pi * h)) * np.exp(2j * np.pi * h * th0)
        k = np.round(h * fr * NF).astype(np.int64) % NF
        g = np.zeros(NF, complex)
        np.add.at(g, k, z)
        Gi = np.fft.ifft(g) * NF
        out += np.imag(Gi[qs % NF])
    tot = out + oS
    for i in np.argsort(tot)[-int(os.environ.get("TOP", 20)) :]:
        best.append((tot[i], R * qs[i] + r))
    if r == 0:
        mu, sd = np.median(tot), tot.std()
best.sort(reverse=True)
print(
    "%s.%d fscan: R %d, k range +-%d, %.0fs; top: %s"
    % (
        CH,
        J,
        R,
        R * M,
        time.time() - t0,
        " | ".join("k=%+d %.1f (Z %.1f)" % (k, v_, (v_ - mu) / sd) for v_, k in best[:5]),
    ),
    flush=True,
)
if mode == "control":
    Nt, Dt = ld(os.environ["TRUTHFILE"])
    Ct = Nt if CH == "N" else Dt
    kt = round((Ct[J] - C[J]) / uc)
    print("   truth k=%+d; in top list: %s" % (kt, [i for i, (v_, k) in enumerate(best) if k == kt]))
np.save(os.path.join(os.environ.get("TMPDIR_W5", "/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad"), "fscan_top_%s%d.npy" % (CH, J)), np.array([[k, v_] for v_, k in best]))
