# T-1941 W6: whole-period scan below a lit sum site. Once ch.Sj is lit at some offset (sitescan.py), c_j is known only
# mod the S_j rounding period: S_j -> S_j + k*PER*u(S_j). The product below, P_{j-1} = fl(S_j x), has no coefficient of
# its own, so its lit k is the period count. Fit beta of the P_{j-1} error regressor for each k (sitemap detrending).
# Usage: pkscan.py RFILE CHAIN CH J KMAX [PER] [npts]   -> top k (c_j shift = k*PER ulps of c_j)   env KC (centre), KEEP (unstored sites above), STAGE=1 (P and S of stage J-1 together), DEPTH=d (stages J-1..J-d), HID=1 (scan a hidden c/x inside P_{J-1} instead: c = k*PER*ulp(S_{J-1}))
import sys, os, numpy as np

sys.path.insert(0, sys.path[0])
from lib6 import st, L
from synth import read_chain, nrm
from siteruns import load_runs, Detrend

rf, chain, ch, J, KM = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5])
PER = float(sys.argv[6]) if len(sys.argv) > 6 else 2.0
npts = int(sys.argv[7]) if len(sys.argv) > 7 else 100000
N, D = read_chain(chain)
N = [v / nrm(D) for v in N]
D = [v / nrm(D) for v in D]
x, lo, hi, gid = load_runs(rf, npts)
detrend = Detrend(x, gid)
XL = x.astype(L)
C = N if ch == "N" else D
s = L(C[-1]) * np.ones_like(XL)
KEEP = set(os.environ.get("KEEP", "").split(",")) - {""}  # sites above the scanned product left unstored (80-bit)
for j in range(len(C) - 2, J - 1, -1):
    p_ = s * XL
    p_ = p_ if "%s.P%d" % (ch, j) in KEEP else L(st(p_))
    s = p_ + L(C[j])
    s = s if "%s.S%d" % (ch, j) in KEEP else L(st(s))
u = L(np.spacing(abs(C[J]))) * np.ones_like(XL)  # PER is in ulps of c_J (the S_J lit period)
Nv = np.polyval(np.array(N, dtype=L)[::-1], XL)
Dv = np.polyval(np.array(D, dtype=L)[::-1], XL)
R = Nv / Dv
y = np.array(((lo.astype(L) + hi.astype(L)) / 2 - R) / R, dtype=float)
w = np.array(XL ** (J - 1) / (Nv if ch == "N" else -Dv), dtype=float)



y = detrend(y)
# env M3=M: also scan c_{J-1} offsets 0..M-1 (ulps) and fit the P_{J-1} and S_{J-1} errors jointly (a coarser S grid
# swallows part of the P error, so P alone can read beta < 1 even when stored); score = Z of the P_{J-1} beta
M3 = int(os.environ.get("M3", "0"))
uc1 = L(np.spacing(abs(C[J - 1]))) if J >= 1 else L(0)
out = []
KC = int(os.environ.get("KC", "0"))  # scan centre (periods)
ks = np.arange(KC - KM, KC + KM + 1)
HID = os.environ.get("HID") == "1"
STAGE = os.environ.get("STAGE") == "1"
DEPTH = int(os.environ.get("DEPTH", "1"))  # >1: score stages J-1..J-DEPTH together (weak sites far above a lit one)  # score the whole stage J-1 (P and S) instead of P_{J-1} alone
uh = float(np.median(np.spacing(np.abs(st(L(st(s * XL)) + L(C[J - 1])))))) if J >= 1 else 0.0
if not M3:
    # batched: rows = k values; detrend each run block by its projector; beta, Z per k
    yy = np.dot(y, y)
    for b0 in range(0, len(ks), 128):
        kb = ks[b0 : b0 + 128]
        if HID:  # hidden hand-off term inside the product rounding: P' = fl(S x + c/x), c = k*PER*ulp(S_{J-1})
            pre = s[None, :] * XL[None, :] + (kb[:, None] * PER).astype(L) * L(uh) / XL[None, :]
        else:
            pre = (s[None, :] + (kb[:, None] * PER).astype(L) * u[None, :]) * XL[None, :]
        if DEPTH > 1:  # stages J-1 .. J-DEPTH recomputed from the shifted S_J; summed weighted rounding errors
            E = np.zeros(pre.shape)
            for j in range(J - 1, max(-1, J - 1 - DEPTH), -1):
                if j < J - 1:
                    pre = sj * XL[None, :]
                pj = L(st(pre))
                vj = pj + L(C[j])
                sj = L(st(vj))
                E += np.array((pj - pre) + (sj - vj), dtype=float) * np.array(XL**j / (Nv if ch == "N" else -Dv), dtype=float)[None, :]
        elif STAGE:  # whole stage J-1: P and S errors together, e = fl(fl(pre) + c_{J-1}) - (pre + c_{J-1})
            E = np.array(L(st(L(st(pre)) + L(C[J - 1]))) - (pre + L(C[J - 1])), dtype=float) * w[None, :]
        else:
            E = np.array(L(st(pre)) - pre, dtype=float) * w[None, :]
        for m, (A, Ai) in zip(detrend.G, detrend.P):
            E[:, m] -= (E[:, m] @ Ai.T) @ A.T
        ee = np.einsum("ij,ij->i", E, E)
        ey = E @ y
        b = ey / ee
        se = np.sqrt((yy - 2 * b * ey + b * b * ee) / len(y) / ee)
        out += list(zip(b / se, b, kb, np.zeros(len(kb), int), np.zeros(len(kb))))
for k in ks if M3 else []:
    pre = (s + L(k * PER) * u) * XL
    p = L(st(pre))
    e = detrend(np.array(p - pre, dtype=float) * w)
    for m in range(M3):
        sv = p + L(C[J - 1]) + L(m) * uc1
        e2 = detrend(np.array(L(st(sv)) - sv, dtype=float) * w)
        A = np.vstack([e, e2]).T
        bb, *_ = np.linalg.lstsq(A, y, rcond=None)
        r = y - A @ bb
        cov = np.mean(r**2) * np.linalg.pinv(A.T @ A)
        se = np.sqrt(np.diag(cov))
        out.append((bb[0] / se[0] + bb[1] / se[1], bb[0], k, m, bb[1]))
out.sort(reverse=True)
uc = np.spacing(abs(C[J]))
for z, b, k, m, b2 in out[:8]:
    print("  k %+6d (c%d %+g ulps)%s  beta %+.3f  Z %.1f" % (k, J, k * PER, ("  c%d +%d  beta2 %+.3f" % (J - 1, m, b2)) if M3 else "", b, z))
print("  k 0: " + " ".join(["m %d beta %+.3f Z %.1f" % (o[3], o[1], o[0]) for o in out if o[2] == 0][:3]))
