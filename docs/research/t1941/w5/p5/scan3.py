"""Global coherent scan of a top stage's smooth delta, with joint re-scoring on dependent sites (T-1941 W5 part 5).

Unknown: dS(x) = alpha + beta*dx + gamma*dx^2 (absolute units, dx = x - X0), the change of a top sum stage S_j
(N.S7 by default). Site phases: S_j itself, phi = dS/u_S; the next product P_{j-1} = S_j*x, phi = dS*x/u_P.
Step 1: FFT scan of the S_j objective over (a = alpha/u_S mod 1, b = beta/u_S, c = gamma/u_S); keep the top
KEEP hypotheses per c. Step 2: joint objective = S_j objective + P_{j-1} objective; report the best.
usage: scan2.py control|real   env: CHAIN (N), J (7), BMAX, CMAX, CSTEP, H, NB, KEEP, SEED, TSIZE"""

import numpy as np, sys, os, time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
from sc import res
from meter import sites_and_weights, template, resid, L

mode = sys.argv[1]
CH = os.environ.get("CHAIN", "N")
J = int(os.environ.get("J", 7))


def ld(f):
    l = open(f).read().split("\n")
    return np.array([float.fromhex(v) for v in l[0].split()]), np.array(
        [float.fromhex(v) for v in l[1].split()]
    )


N0, D0 = ld(os.environ.get("BASE", os.path.join(HERE, "..", "p4", "ss_fc.txt")))
# data: fresh rows (includes e9xx) cached as npz
ROWS = os.environ.get(
    "ROWS", "/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows2.txt"
)
cache = ROWS + ".near1.npz"
if os.path.exists(cache):
    z = np.load(cache)
    x = z["x"]
    b = z["b"]
else:
    X = []
    B = []
    for l in open(ROWS):
        xh, bh, _, _, nm = l.split()
        if bh == "nan":
            continue
        xv = float.fromhex(xh)
        if 0.7 <= xv < 1.5:
            X.append(xv)
            B.append(float.fromhex(bh))
    x = np.array(X)
    b = np.array(B)
    x, ui = np.unique(x, return_index=True)
    b = b[ui]
    np.savez(cache, x=x, b=b)
X0 = 1.1
LO, HI = float(os.environ.get("LO", 0.7)), float(os.environ.get("HI", 1.47))
RL = int(os.environ.get("RL", 4096))
br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0] + 1
segs = [
    s[:RL]
    for s in np.split(np.arange(len(x)), br)
    if len(s) >= 256 and LO <= x[s[0]] < HI and abs(x[s[0]] - 1) > 2e-3
]
if mode == "control":
    from synth import make_truth
    if os.environ.get('TRUTHFILE'):
        _l = open(os.environ['TRUTHFILE']).read().split('\n')
        make_truth = lambda *a: (np.array([float.fromhex(v) for v in _l[0].split()]), np.array([float.fromhex(v) for v in _l[1].split()]), np.zeros(19), 0.0)

    Nt, Dt, mv, fc = make_truth(N0, D0, int(os.environ.get("SEED", 1)), float(os.environ.get("TSIZE", 1e5)))
    print("synthetic move", mv.astype(int).tolist(), "func change %.1e" % fc)
kS = (CH, "S", J)
kP = (CH, "P", J - 1)


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
tS = []
t0 = time.time()
for s in segs:
    xx = x[s]
    W, ref = sites_and_weights(N0, D0, xx)
    Bv = res(xx, Nt, Dt) if mode == "control" else b[s]
    r = resid(xx, Bv, ref)
    c1, u1 = curves(kS, W, r)
    c2, u2 = curves(kP, W, r)
    CS.append(c1)
    CP.append(c2)
    uS.append(u1)
    uP.append(u2)
    xm.append(xx[len(xx) // 2])
    if mode == "control":
        Wt, _ = sites_and_weights(Nt, Dt, xx)
        pre = W[kS][0]
        tS.append(float(np.median((Wt[kS][0] - pre).astype(float))))
CS = np.array(CS)
CP = np.array(CP)
xm = np.array(xm)
uS = np.array(uS)
uP = np.array(uP)
R = len(xm)
dx = xm - X0
print("runs %d prep %.0fs; u_S values %s" % (R, time.time() - t0, np.unique(uS)), flush=True)
u0 = float(np.median(uS))
truth = None
if mode == "control":
    tp = np.polyfit(dx, np.array(tS), 3)
    truth = (tp[3] / u0, tp[2] / u0, tp[1] / u0, tp[0] / u0)
    print('truth cubic coefficient d = %.3f' % (tp[0] / u0))
    print(
        "truth (alpha, beta, gamma)/u_S = (%.3f, %.3f, %.3f), a mod 1 = %.3f"
        % (truth[0], truth[1], truth[2], truth[0] % 1)
    )


def obj(C, ph):
    ph = ph % 1.0
    k = np.floor(ph * 64).astype(int) % 64
    fr = ph * 64 - np.floor(ph * 64)
    idx = np.arange(C.shape[0])
    return (
        (C[idx, k] * (1 - fr) + C[idx, (k + 1) % 64] * fr).sum(-1)
        if ph.ndim == 1
        else (C[idx[None, :], k] * (1 - fr) + C[idx[None, :], (k + 1) % 64] * fr).sum(-1)
    )


def joint(a, bb, cc, dd=0.0):
    dS = (a + bb * dx + cc * dx**2 + dd * dx**3) * u0  # absolute
    return obj(CS, dS / uS), obj(CP, dS * xm / uP)


if truth:
    s1, s2 = joint(*truth)
    print("at truth: S %.1f  P %.1f  joint %.1f" % (s1, s2, s1 + s2))
    if os.environ.get("PERRUN"):
        # per-run truth contribution and per-run spread over phase (where the top-stage information lives)
        phr = ((truth[0] + truth[1] * dx + truth[2] * dx**2 + truth[3] * dx**3) * u0 / uS) % 1.0
        kk = np.floor(phr * 64).astype(int) % 64
        np.savez(os.environ["PERRUN"], xm=xm, tv=CS[np.arange(len(xm)), kk], sd=CS.std(1))
        raise SystemExit(0)
# null for the joint objective
rng = np.random.default_rng(3)
nul = np.array(
    [sum(joint(rng.random(), rng.uniform(-2e4, 2e4), rng.uniform(-200, 200))) for _ in range(5000)]
)
print("joint null mean %.1f sd %.1f" % (nul.mean(), nul.std()), flush=True)
# FFT scan on the S objective
H = int(os.environ.get("H", 6))
Ch = np.fft.fft(CS, axis=1) / CS.shape[1]
BMAX = float(os.environ.get("BMAX", 2e4))
CMAX = float(os.environ.get("CMAX", 200))
CSTEP = float(os.environ.get("CSTEP", 0.05))
NB = 1 << int(os.environ.get("NB", 21))
KEEP = int(os.environ.get("KEEP", 40))
span = dx.max() - dx.min() + 1e-9
dx0 = dx.min()
cell = span / NB * 4
kbin = np.round((dx - dx0) / cell).astype(int)
bgrid = np.fft.fftfreq(NB, d=cell)
keepb = np.abs(bgrid) <= BMAX
bsel = bgrid[keepb]
f = np.round(bsel * NB * cell).astype(int)
ag = np.arange(64) / 64.0
cand = []
t0 = time.time()
cgrid = np.arange(-CMAX, CMAX + CSTEP / 2, CSTEP)
if truth and os.environ.get("NEAR"):
    cgrid = np.arange(truth[2] - 1, truth[2] + 1, CSTEP)
dgrid = np.arange(-float(os.environ.get('DMAX', 0)), float(os.environ.get('DMAX', 0)) + 1e-9, float(os.environ.get('DSTEP', 0.5)))
for dd, cc in [(dd_, cc_) for dd_ in dgrid for cc_ in cgrid]:
    tot = np.zeros((64, len(bsel)))
    for h in range(1, H + 1):
        zz = np.zeros(NB, complex)
        np.add.at(zz, kbin, Ch[:, h] * np.exp(2j * np.pi * h * (cc * dx**2 + dd * dx**3)))
        G = np.fft.ifft(zz) * NB
        Gh = G[(h * f) % NB] * np.exp(2j * np.pi * h * bsel * dx0)
        tot += 2 * np.real(np.exp(2j * np.pi * h * ag)[:, None] * Gh[None, :])
    ia = tot.argmax(0)
    vals = tot[ia, np.arange(len(bsel))]
    for i in np.argpartition(vals, -KEEP)[-KEEP:]:
        cand.append((vals[i], ag[ia[i]], bsel[i], cc, dd))
print("scan %.0fs, %d candidates" % (time.time() - t0, len(cand)), flush=True)
cand.sort(reverse=True)
print("top S-only:", " | ".join("%.1f (a %.2f b %.1f c %.2f d %.2f)" % c for c in cand[:5]))
np.save(os.path.join(os.environ.get("TMPDIR_W5", "/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad"), "scan3_%s%d_%s.npy" % (CH, J, mode)), np.array(cand[:2000]))
if truth: print("truth a mod 1 %.3f b %.2f c %.2f" % (truth[0] % 1, truth[1], truth[2]))
