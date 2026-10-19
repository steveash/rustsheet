# T-1941 W6: W5's global coherent top-stage scan (w5/p5/scan2.py) on W6 data, for a LINEAR top stage delta (S8 of a
# degree-9 raw-x Horner chain: S8 = c8 + c9 x, so dS8(x) = (a + b dx) u_S, dx = x - X0). Per run of consecutive
# doubles: the correlation of the R residual with the S_J site's rounding template at 64 phases (W5's meter.template)
# and the same for the next product P_{J-1}. Step 1: FFT over b for every a (mod 1, 64-grid), H harmonics of the
# sawtooth; Step 2: re-score the top candidates jointly on P_{J-1}. Phase-coherent over all runs at once, so there is
# no capture-range limit (W5). Usage: scan6.py RFILE BASE CH J [TRUTH]
#   env BMAX (1e7), H (4), RL (run chunk, 4096), KEEP (200), X0 (1.75), NPROBE (5000 null draws)
import sys, os, re, time, collections, numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.append(os.path.join(HERE, "..", "w5", "p5"))  # after w6: both have a synth.py
from meter import chain_sites, template, L
from synth import read_chain

rf, base, CH, J = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
truthf = sys.argv[5] if len(sys.argv) > 5 else None
X0 = float(os.environ.get("X0", 1.75))
DEG = int(os.environ.get("DEG", 1))  # 1: linear top stage (S8); 2: quadratic (S7); 3: cubic (S6), EVALONLY
RL = int(os.environ.get("RL", 4096))
N, D = read_chain(base)
N = np.array(N) / D[0]
D = np.array(D) / D[0]
C = N if CH == "N" else D
if truthf:
    Nt, Dt = read_chain(truthf)
    Ct = (np.array(Nt) if CH == "N" else np.array(Dt)) / Dt[0]
pts = collections.defaultdict(list)
for l in open(rf):
    q = l.split()
    pts[re.sub(r"[a-d]?(:B)?$", "", q[5])].append(
        (float.fromhex(q[0]), float.fromhex(q[2]), float.fromhex(q[3]))
    )
segs = []
for v in pts.values():
    v = np.array(sorted(v))
    br = np.where(np.nextafter(v[:-1, 0], np.inf) != v[1:, 0])[0] + 1
    for s in np.split(np.arange(len(v)), br):
        for i in range(0, len(s) - 255, RL):
            segs.append(v[s[i : i + RL]])


def curve(pre, sv, w, r):
    T = template(pre, sv, w)
    Tc = T - T.mean(1, keepdims=True)
    nrm = np.sqrt((Tc * Tc).sum(1))
    return (Tc @ r) / np.maximum(nrm, 1e-30) / max(np.sqrt(r @ r), 1e-30) * np.sqrt(len(r)), float(
        np.median(np.spacing(np.abs(sv)))
    )


CS, CP, xm, uS, uP, tS = [], [], [], [], [], []
t0 = time.time()
for v in segs:
    x = v[:, 0]
    XL = x.astype(L)
    Nv = np.polyval(N.astype(L)[::-1], XL)
    Dv = np.polyval(D.astype(L)[::-1], XL)
    R = Nv / Dv
    r = (((v[:, 1].astype(L) + v[:, 2].astype(L)) / 2 - R) / (np.abs(R) * L(2.0**-52))).astype(float)
    r -= r.mean()
    sites, _ = chain_sites(C, x)
    wf = (1 / Nv) if CH == "N" else (-1 / Dv)
    preS, svS = sites[("S", J)]
    preP, svP = sites[("P", J - 1)]
    c1, u1 = curve(preS, svS, wf * XL**J, r)
    c2, u2 = curve(preP, svP, wf * XL ** (J - 1), r)
    CS.append(c1)
    CP.append(c2)
    uS.append(u1)
    uP.append(u2)
    xm.append(x[len(x) // 2])
    if truthf:
        st_, _ = chain_sites(Ct, x)
        tS.append(float(np.median((st_[("S", J)][0] - preS).astype(float))))
CS, CP, xm, uS, uP = (np.array(a) for a in (CS, CP, xm, uS, uP))
dx = xm - X0
u0 = float(np.median(uS))
print(
    "runs %d (chunk %d), prep %.0fs; u_S values %s" % (len(xm), RL, time.time() - t0, np.unique(uS)),
    flush=True,
)


def obj(Cm, ph):
    ph = ph % 1.0
    k = np.floor(ph * 64).astype(int) % 64
    fr = ph * 64 - np.floor(ph * 64)
    idx = np.arange(Cm.shape[0])
    if ph.ndim == 1:
        return (Cm[idx, k] * (1 - fr) + Cm[idx, (k + 1) % 64] * fr).sum(-1)
    return (Cm[idx[None, :], k] * (1 - fr) + Cm[idx[None, :], (k + 1) % 64] * fr).sum(-1)


def joint(a, b, c=0.0, *hi):
    dS = (a + b * dx + c * dx**2 + sum(h * dx ** (3 + k) for k, h in enumerate(hi))) * u0
    return obj(CS, dS / uS), obj(CP, dS * xm / uP)


truth = None
if truthf:
    tp = np.polyfit(dx, np.array(tS), DEG)[::-1] / u0
    truth = tuple(tp) if DEG >= 2 else (tp[0], tp[1], 0.0)
    s1, s2 = joint(*truth)
    print(
        "truth (a, b, c..) = (%.3f, %.1f, %s), a mod 1 %.3f; at truth: S %.1f  P %.1f  joint %.1f; fit rms %.3f u"
        % (truth[0], truth[1], ", ".join("%.2f" % v for v in truth[2:]), truth[0] % 1, s1, s2, s1 + s2, np.std(np.array(tS) / u0 - np.polyval(tp[::-1], dx))),
        flush=True,
    )
BMAX = float(os.environ.get("BMAX", 1e7))
CMAX = float(os.environ.get("CMAX", 0))
DMAX = float(os.environ.get("DMAX", 0))
rng = np.random.default_rng(3)
nul = np.array(
    [sum(joint(rng.random(), rng.uniform(-BMAX, BMAX), rng.uniform(-CMAX, CMAX), rng.uniform(-DMAX, DMAX))) for _ in range(int(os.environ.get("NPROBE", 5000)))]
)
print("joint null mean %.1f sd %.1f" % (nul.mean(), nul.std()), flush=True)
if os.environ.get("EVALONLY"):
    raise SystemExit(0)
H = int(os.environ.get("H", 4))
KEEP = int(os.environ.get("KEEP", 200))
Ch = np.fft.fft(CS, axis=1) / CS.shape[1]
span = dx.max() - dx.min() + 1e-9
dx0 = dx.min()
NB = 1 << int(np.ceil(np.log2(max(1 << 10, 2 * BMAX * span * 4 * 1.1))))
cell = span / NB * 4
kbin = np.round((dx - dx0) / cell).astype(int)
bgrid = np.fft.fftfreq(NB, d=cell)
keepb = np.abs(bgrid) <= BMAX
bsel = bgrid[keepb]
f = np.round(bsel * NB * cell).astype(np.int64)
t0 = time.time()
# explicit grids over the higher terms (c for DEG >= 2, d for DEG 3), FFT over b for each, batched
cg = np.arange(-CMAX, CMAX + 1e-9, float(os.environ.get("CSTEP", 0.5))) if DEG >= 2 else np.array([0.0])
dg = np.arange(-DMAX, DMAX + 1e-9, float(os.environ.get("DSTEP", 2.0))) if DEG >= 3 else np.array([0.0])
if truth and os.environ.get("NEAR"):  # control only: grids around the truth (validation of the search itself)
    w_ = float(os.environ["NEAR"])
    cg = cg[np.abs(cg - truth[2]) <= w_ * float(os.environ.get("CSTEP", 0.5))] if DEG >= 2 else cg
    dg = dg[np.abs(dg - (truth[3] if DEG >= 3 else 0)) <= w_ * float(os.environ.get("DSTEP", 2.0))] if DEG >= 3 else dg
pairs = np.array([(c_, d_) for c_ in cg for d_ in dg])
BATCH = int(os.environ.get("BATCH", 128))
ph_b = np.exp(2j * np.pi * np.outer(np.arange(1, H + 1), bsel * dx0))
import scipy.sparse as sps
Smap = sps.csr_matrix((np.ones(len(kbin)), (np.arange(len(kbin)), kbin)), shape=(len(kbin), NB)).astype(complex)
cand = []
for i0 in range(0, len(pairs), BATCH):
    P_ = pairs[i0 : i0 + BATCH]
    hi = np.outer(P_[:, 0], dx**2) + np.outer(P_[:, 1], dx**3)  # (batch, runs)
    Ghs = []
    for h in range(1, H + 1):
        val = Ch[None, :, h] * np.exp(2j * np.pi * h * hi)
        zz = (val @ Smap).toarray() if hasattr(val @ Smap, "toarray") else val @ Smap  # runs -> b-cells (sparse)
        G = np.fft.ifft(zz, axis=1) * NB
        Ghs.append((G[:, (h * f) % NB] * ph_b[h - 1][None, :]).astype(np.complex64))
    bv = np.full(Ghs[0].shape, -np.inf, dtype=np.float32)
    ba = np.zeros(Ghs[0].shape, dtype=np.float32)
    for ia in range(64):
        tot = np.zeros(Ghs[0].shape, dtype=np.float32)
        for h in range(1, H + 1):
            tot += 2 * np.real(np.complex64(np.exp(2j * np.pi * h * ia / 64)) * Ghs[h - 1])
        m = tot > bv
        bv[m] = tot[m]
        ba[m] = ia / 64
    flat = bv.ravel()
    k_ = min(KEEP, flat.size)
    for j in np.argpartition(flat, -k_)[-k_:]:
        pi_, bi_ = divmod(j, bv.shape[1])
        cand.append((float(flat[j]), float(ba.ravel()[j]), float(bsel[bi_]), float(P_[pi_, 0]), float(P_[pi_, 1])))
    if len(cand) > 20 * KEEP:
        cand.sort(reverse=True)
        cand = cand[:KEEP]
    if i0 // BATCH % 50 == 0:
        print("  %d/%d (c, d) pairs, %.0fs, best so far %.1f" % (i0 + len(P_), len(pairs), time.time() - t0, max(c[0] for c in cand)), flush=True)
cand.sort(reverse=True)
cand = cand[:KEEP]
print("scan: NB 2^%d, %d b values (spacing %.3f), %d (c, d) pairs, %.0fs" % (int(np.log2(NB)), len(bsel), bgrid[1], len(pairs), time.time() - t0), flush=True)
# refine locally and re-score jointly with the P site
best = []
cs_, ds_ = float(os.environ.get("CSTEP", 0.5)), float(os.environ.get("DSTEP", 2.0))
for v0, a, b, c_, d_ in cand:
    g = []
    for x_ in a + np.linspace(-0.5, 0.5, 5) / 64:
        for y_ in b + np.linspace(-0.5, 0.5, 7) * bgrid[1]:
            for z_ in (c_ + np.linspace(-0.5, 0.5, 3) * cs_ if DEG >= 2 else [0.0]):
                for w_ in (d_ + np.linspace(-0.5, 0.5, 3) * ds_ if DEG >= 3 else [0.0]):
                    g.append((sum(joint(x_, y_, z_, w_)), x_, y_, z_, w_))
    best.append(max(g))
best.sort(reverse=True)
print("top joint (Z vs null):")
for v, a, b, c_, d_ in best[:8]:
    print("  joint %.1f (Z %.1f)  a %.3f  b %.2f  c %.2f  d %.2f" % (v, (v - nul.mean()) / nul.std(), a % 1, b, c_, d_), flush=True)
if truth:
    print("truth: a mod 1 %.3f  b %.2f  %s" % (truth[0] % 1, truth[1], "  ".join("%.2f" % v for v in truth[2:])))
if os.environ.get("OUT"):
    np.save(os.environ["OUT"], np.array(best[:50]))
