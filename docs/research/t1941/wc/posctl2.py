# positive control v2: phase rows + measured-function rows (bin means over the 2.27M grid), two-scale.
# usage: A2=1 MORE=1 FREEZE=8,17,9 python3 posctl2.py AN2.txt 3e5 3 ITERS 0.005 5e-9
import sys, pickle, os

exec(open("jointA.py").read().replace("if __name__=='__main__':", "if False:"))
x0 = np.array([float.fromhex(h) for h in open(sys.argv[1]).read().split()])
target_ulps = float(sys.argv[2])
ndir = int(sys.argv[3])
iters = int(sys.argv[4])
PH = float(sys.argv[5])
FZ = [8, 17, 9]
keep = [c for c in range(18) if c not in FZ]
d = np.load("grows.npz")
Yg = d["y"]
Ghi = d["gam"]
Glo = d["gamlo"]
SPg = np.spacing(Ghi)
mp.mp.dps = 40
ysv = [mp.mpf(1) + mp.mpf(k) / 200 for k in range(201)]
R = []
for t in ysv:
    gg = mp.gamma(t)
    Dv = sum(mp.mpf(x0[9 + i]) * t**i for i in range(9))
    R.append([float(t**i / (gg * Dv)) for i in range(9)] + [float(-(t**i) / Dv) for i in range(9)])
R = np.array(R)[:, keep]
sc = np.abs(x0[keep])
U, S, Vt = np.linalg.svd(R * sc, full_matrices=False)
rng = np.random.default_rng(5)
dx = np.zeros(18)
for k in range(1, ndir + 1):
    dd = np.zeros(18)
    dd[keep] = Vt[-k] * sc
    dx += rng.normal() * dd
ulps = np.array([dx[i] / math.ulp(x0[i]) for i in range(18)])
dx *= target_ulps / np.max(np.abs(ulps))
xt = np.array([float(Fr(x0[i]) + Fr(dx[i])) for i in range(18)])
NB = 100
bidx = np.minimum(((Yg - 1) * NB).astype(int), NB - 1)


def binstats(q):
    e = (q - Ghi) / SPg - Glo
    cnt = np.bincount(bidx, minlength=NB)
    s = np.bincount(bidx, e, minlength=NB)
    s2 = np.bincount(bidx, e * e, minlength=NB)
    m = s / np.maximum(cnt, 1)
    var = s2 / np.maximum(cnt, 1) - m * m
    se = np.sqrt(np.maximum(var, 1e-6) / np.maximum(cnt, 1))
    yc = np.bincount(bidx, Yg, minlength=NB) / np.maximum(cnt, 1)
    return m, se, yc, cnt


qt = model(xt, Yg)
mt, set_, yc, cnt = binstats(qt)
ok = cnt > 200
print("bins used", ok.sum(), " median SE %.4f ulp" % np.median(set_[ok]))


def func_rows(x):
    q = model(x, Yg)
    mx, _, _, _ = binstats(q)
    rows = []
    res = []
    for b in np.where(ok)[0]:
        t = mp.mpf(yc[b])
        Nv = sum(mp.mpf(x[i]) * t**i for i in range(9))
        Dv = sum(mp.mpf(x[9 + i]) * t**i for i in range(9))
        F = Nv / Dv
        gg = mp.gamma(t)
        u = mp.mpf(np.spacing(float(gg)))
        fdev = float((F - gg) / u)
        rho = mx[b] - fdev
        J = [float(t**i / Dv / u) for i in range(9)] + [float(-F * t**i / Dv / u) for i in range(9)]
        w = PH / set_[b]
        rows.append(np.array(J) * w)
        res.append((mt[b] - rho - fdev) * w)
    return np.array(rows), np.array(res)


e7 = pickle.load(open("e7e10.pkl", "rb"))
runs = load_e3() + e7
open("pc_truth.txt", "w").write(" ".join(float(v).hex() for v in xt) + "\n")
syn = []
for a, yv, g, rd in runs:
    q = model(xt, yv)
    syn.append((a, yv, q, rd + (q - g) / ULP))


def exact(x):
    return float(np.mean(model(x, Yg) == qt))


print("exact(x0 vs truth on grid) %.4f" % exact(x0), flush=True)
x = x0.copy()
globals()["RIDGE"] = float(sys.argv[6]) if len(sys.argv) > 6 else None
for it in range(iters):
    globals()["CUSTOM_F"] = func_rows(x)
    dd = solve_joint(x, syn, log=False)
    x = apply(x, dd)
    err = np.array([(x[i] - xt[i]) / math.ulp(xt[i]) for i in range(18)])
    open("pc_iter%d.txt" % it, "w").write(" ".join(float(v).hex() for v in x) + "\n")
    print(
        "iter %d: exact %.4f  max|x-truth| %.3g ulps: %s"
        % (it, exact(x), np.max(np.abs(err)), " ".join("%.3g" % e for e in err)),
        flush=True,
    )
