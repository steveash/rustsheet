# T-1941 W6: W5's FFT peel scan (w5/p5/fscan.py) ported to the W6 chain and R data. Scores every integer offset k
# (ulps) of one coefficient c_J by the correlation of the rounding errors it creates with the data residual, so it is
# informative while other coefficients are still off (an exact-match rate is not).
#   r_p   = (R_data - N/D of the base chain, long double) / (|R| 2^-52), zero mean per run of consecutive doubles
#   k = Rr*q + r (Rr = ulp(S_J)/ulp(c_J), a power of two): the S_J = fl(P_J + c_J + k uc) rounding error eS depends on
#   r only; the product P_{J-1} = fl(S_J x) error is a sawtooth in q, scored for all q at once by harmonics + FFT.
#   objective(k) = sum_p wS eS r_p + sum_p wP eP r_p, wS = x^J/Nv (N) or -R x^J/Dv (D), wP = the same at J-1.
# Usage: fscan6.py RFILE CHAIN CH J [KRANGE] [TRUTH]   env H (harmonics, 6), TOP (20), NPTS (all), CENTER (0)
import sys, os, re, collections, numpy as np

sys.path.insert(0, sys.path[0])
from lib6 import st, L
from synth import read_chain

rf, chain, CH, J = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
KR = float(sys.argv[5]) if len(sys.argv) > 5 else 2e6
N, D = read_chain(chain)
N = np.array(N) / D[0]
D = np.array(D) / D[0]
# data: runs of consecutive doubles from RFILE (rows "x B Rlo Rhi n case"); residual vs the base chain's exact rational
runs = collections.defaultdict(list)
for l in open(rf):
    q = l.split()
    runs[re.sub(r"[a-d]?(:B)?$", "", q[5])].append(
        (float.fromhex(q[0]), float.fromhex(q[2]), float.fromhex(q[3]))
    )
xs, rs = [], []
for k, v in runs.items():
    v = np.array(sorted(v))
    x = v[:, 0]
    XL = x.astype(L)
    Nv = np.polyval(N.astype(L)[::-1], XL)
    Dv = np.polyval(D.astype(L)[::-1], XL)
    Rm = (v[:, 1].astype(L) + v[:, 2].astype(L)) / 2
    r = ((Rm - Nv / Dv) / (np.abs(Nv / Dv) * L(2.0**-52))).astype(float)
    xs.append(x)
    rs.append(r - r.mean())
x = np.concatenate(xs)
r = np.concatenate(rs)
if os.environ.get("NPTS"):
    i = np.random.default_rng(0).choice(len(x), int(os.environ["NPTS"]), replace=False)
    x, r = x[i], r[i]
XL = x.astype(L)
Nv = np.polyval(N.astype(L)[::-1], XL)
Dv = np.polyval(D.astype(L)[::-1], XL)
C = N if CH == "N" else D
w = (
    (1 / Nv) if CH == "N" else (-(Nv / Dv) / Dv / (Nv / Dv))
)  # relative-R weight per absolute error at stage 0
wS = (w * XL**J).astype(float) * 2.0**52
wP = (w * XL ** (J - 1)).astype(float) * 2.0**52 if J > 0 else np.zeros(len(x))
uc = float(np.spacing(abs(C[J])))
CEN = int(float(os.environ.get("CENTER", "0")))  # scan window centre (ulps of c_J from the base value)
C = C.copy()
C[J] = C[J] + CEN * uc
# stored chain down to P_J (stages above J at the base values)
s = np.full(len(x), C[-1])
PJ = None
for j in range(len(C) - 2, J - 1, -1):
    p = st(L(s) * L(x))
    if j == J:
        PJ = p
        break
    s = st(L(p) + L(C[j]))
if PJ is None:  # J is the top coefficient
    PJ = np.zeros(len(x))
S0 = st(L(PJ) + L(C[J]))
uS = np.spacing(np.abs(S0))
Rr = int(max(1, 2 ** np.ceil(np.log2(max(1.0, (uS / uc).max())))))
M = int(KR / Rr) + 1
H = int(os.environ.get("H", 6))
NS = int(os.environ.get("NS", "2"))  # downstream stages modelled
NF = 1 << int(min(25, max(18, np.ceil(np.log2(M * 64)))))
qs = np.arange(-M, M + 1)
best = []
for rr in range(Rr):
    se = L(PJ) + L(C[J]) + L(rr) * L(uc)
    Sr = st(se)
    eS = (L(Sr) - se).astype(float)
    oS = float(np.sum(wS * eS * r))
    out = np.zeros(len(qs))
    # downstream rounding sites (products and sums of stages J-1 .. J-NS): shifting c_J by Rr*uc per q-step moves the
    # exact pre-rounding value at stage i by Rr*uc*x^(J-i), so each site's error is a sawtooth in q (W5 modelled only
    # the first product; a tiny top coefficient is visible only further down, e.g. N9 through S8)
    sprev = Sr
    for i in range(J - 1, max(-1, J - 1 - NS), -1):
        vp = L(sprev) * L(x)
        P = st(vp)
        vs = L(P) + L(C[i])
        sprev = st(vs)
        for v in (vp, vs):
            u_ = np.spacing(np.abs(st(v))).astype(L)
            th0 = v / u_
            th0 = (th0 - np.floor(th0)).astype(float)
            fr = (L(Rr * uc) * XL ** (J - i) / u_).astype(float)
            a = (w * XL**i).astype(float) * 2.0**52 * r * u_.astype(float)
            for h in range(1, H + 1):
                z = a * ((-1) ** h / (np.pi * h)) * np.exp(2j * np.pi * h * th0)
                kk = np.round(h * fr * NF).astype(np.int64) % NF
                g = np.zeros(NF, complex)
                np.add.at(g, kk, z)
                Gi = np.fft.ifft(g) * NF
                out += np.imag(Gi[qs % NF])
    tot = out + oS
    for i in np.argsort(tot)[-int(os.environ.get("TOP", 20)) :]:
        best.append((tot[i], Rr * qs[i] + rr + CEN))
    if rr == 0:
        mu, sd = np.median(tot), tot.std()
best.sort(reverse=True)
print(
    "%s%d fscan6: n %d, Rr %d, k range +-%d; top: %s"
    % (
        CH,
        J,
        len(x),
        Rr,
        Rr * M,
        " | ".join("k=%+d %.1f (Z %.1f)" % (k, v_, (v_ - mu) / sd) for v_, k in best[:5]),
    ),
    flush=True,
)
if len(sys.argv) > 6:
    Nt, Dt = read_chain(sys.argv[6])
    Ct = (np.array(Nt) if CH == "N" else np.array(Dt)) / Dt[0]
    kt = round((Ct[J] - C[J]) / uc) + CEN
    print("   truth k=%+d; rank in top list: %s" % (kt, [i for i, (v_, k) in enumerate(best) if k == kt]))
