# T-1941 W6: are a chain's measured line phases consistent with the stored-Horner model at all? Small CVP over only
# the coefficients the chosen lines depend on (e.g. N4..N9 for N.T4, N.T5, N.Pn14, N.Pn15): integer offsets m_k
# (ulps), rows = measured phase congruences (latsolve.phase_rows gradients, mod 1, sigma SP) + ladder frequency
# congruences (freq_rows, sigma SL). If a short solution leaves phase residuals at the noise level the phases are
# consistent; if not, the phase model (or the measurement) is wrong for those lines. No function rows (they would mix
# in the other chain). Usage:
#   phasecons.py CHAIN PHASES LADDER "N.T4,N.T5,N.Pn14,N.Pn15" "N4,N9" [TRUTH]   env SP (0.03), SL (2e-10), BOUND (1e8)
import sys, os, numpy as np, mpmath as mp
from fpylll import IntegerMatrix, LLL, BKZ

sys.path.insert(0, sys.path[0])
from latsolve import phase_rows, freq_rows, ulp
from synth import read_chain
from stagelaws import stage_F

mp.mp.dps = 60
chain, phf, ladf, lines, krange = sys.argv[1:6]
N, D = read_chain(chain)
N = [v / D[0] for v in N]
D = [v / D[0] for v in D]
nN = len(N)
want = set(lines.split(","))
k0, k1 = [int(v[1:]) for v in krange.split(",")]
ch = krange[0]
cols = [(k if ch == "N" else nN - 1 + k) for k in range(k0, k1 + 1)]
SP, SL, BOUND = (
    float(os.environ.get("SP", 0.03)),
    float(os.environ.get("SL", 2e-10)),
    float(os.environ.get("BOUND", 1e8)),
)
rows = []  # (gradient over cols, target mod 1, sigma, kind)
for l in open(phf):
    q = l.split()
    key = "%s.%s%s" % (q[2], q[3], q[4])
    if key not in want or not (0.7 < float(q[6]) < 1.4):
        continue
    g, _ = phase_rows(N, D, [(q[2], q[3], int(q[4]))], [float(q[1])])[0]
    rows.append(([g[c] for c in cols], mp.mpf(q[5]) % 1, SP, "ph " + key))
for l in open(ladf):
    q = l.split()
    key = "%s.%s%s" % (q[1], q[2], q[3])
    if key not in want or float(q[5]) >= 0.15:
        continue
    c = np.array(N) if q[1] == "N" else np.array(D)
    kk = ("T%s" % q[3]) if q[2] == "T" else "P%sn1" % q[3]
    Fc = stage_F(c, np.array([float(q[0])]))[kk][0][0]
    if abs(((float(q[4]) - Fc + 0.5) % 1 - 0.5) * 65536) > 1.5:
        continue
    g = freq_rows(N, D, [(q[1], q[2], int(q[3]))], [float(q[0])])[0]
    rows.append(([g[c_] for c_ in cols], (mp.mpf(q[4]) - mp.mpf(Fc)) % 1, SL, "F  " + key))
nu, ne = len(cols), len(rows)
print(
    "unknowns %s%d..%s%d, rows: %d phase + %d ladder"
    % (ch, k0, ch, k1, sum(r[3].startswith("ph") for r in rows), sum(r[3].startswith("F") for r in rows))
)
SC = mp.mpf(2) ** 70
dim = nu + ne + 1
M = [[0] * (nu + ne + 1) for _ in range(dim)]
for k in range(nu):
    M[k][k] = int(SC / BOUND)
    for e, (g, _, s, _) in enumerate(rows):
        M[k][nu + e] = int(mp.nint(SC * g[k] / s))
for e, (_, _, s, _) in enumerate(rows):
    M[nu + e][nu + e] = int(mp.nint(SC / s))
for e, (_, t, s, _) in enumerate(rows):
    M[dim - 1][nu + e] = int(mp.nint(-SC * t / s))
M[dim - 1][-1] = int(SC)
A = IntegerMatrix.from_matrix(M)
LLL.reduction(A, method="proved", float_type="mpfr", precision=300)
BKZ.reduction(A, BKZ.Param(block_size=min(30, dim)), float_type="mpfr", precision=300)
best = None
for i in range(A.nrows):
    r = [A[i, j] for j in range(A.ncols)]
    if abs(r[-1]) == int(SC):
        sg = 1 if r[-1] > 0 else -1
        best = [int(round(float(sg * mp.mpf(r[k]) / int(SC / BOUND)))) for k in range(nu)]
        break
print("m (%s%d..%s%d):" % (ch, k0, ch, k1), best)
if best:
    res = {}
    for g, t, s, kind in rows:
        v = float((sum(g[k] * best[k] for k in range(nu)) - t + mp.mpf(0.5)) % 1 - mp.mpf(0.5))
        res.setdefault(kind, []).append(v / (s if kind.startswith("F") else 1))
    for kind, v in sorted(res.items()):
        v = np.array(v)
        print(
            "  %-12s n %2d  residual rms %.3f %s"
            % (kind, len(v), np.sqrt(np.mean(v**2)), "(sigma units)" if kind.startswith("F") else "cycles")
        )
if len(sys.argv) > 6:
    Nt, Dt = read_chain(sys.argv[6])
    Ct = (np.array(Nt) if ch == "N" else np.array(Dt)) / Dt[0]
    C = np.array(N) if ch == "N" else np.array(D)
    print(
        "truth m:", [int(round(float((mp.mpf(Ct[k]) - mp.mpf(C[k])) / ulp(C[k])))) for k in range(k0, k1 + 1)]
    )
