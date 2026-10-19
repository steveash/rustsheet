"""K3 plateau reader (T-1941 W5 part 4).

Objective: per-run mean relative bias of the target (Excel B, or synthetic) minus the candidate's exact
simulation, in relative ulps (2^-52 of |ref|), minus a weighted smooth polynomial fit (degree PDEG in x, the
function difference), summed as chi^2 over runs. A stored sum S_k = fl(P_k + c_k) contributes a constant per
binade configuration that depends on c_k's bits below ulp(S_k) (and on P_k's grid), so low-bit moves of c_k
change the staircase but the smooth part absorbs the tiny function change.

Search: for each constant c_k, moves j*ulp(c_k), |j| up to 2^m with 2^m*ulp(c_k) ~ 4*ulp(S_k); keep the best.
usage: plateau.py control|real [SWEEPS]"""

import numpy as np, sys, os

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
from sc import candidate, load, res, score
from metrics import XF, BF, XN, BN

LO, HI = 0.7, float(os.environ.get("HI", 1.47))
PDEG = int(os.environ.get("PDEG", 8))
x, b, c = load()
m = (x >= LO) & (x < HI) & (np.abs(x - 1) > 2e-3)  # |x-1| tiny: relative bias meaningless (t*A rounding)
x = x[m]
b = b[m]
# subsample up to PER points per run (same points for target and candidate)
_br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0] + 1
_keep = np.concatenate([seg[np.linspace(0, len(seg) - 1, min(len(seg), int(os.environ.get('PER', 64)))).astype(int)] for seg in np.split(np.arange(len(x)), _br)])
x = x[_keep]; b = b[_keep]
br = np.where(np.diff(x) > 1e-9)[0] + 1
rid = np.zeros(len(x), int)
rid[br] = 1
rid = np.cumsum(rid)
R = rid[-1] + 1
cnt = np.bincount(rid)
xr = np.bincount(rid, x) / cnt
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


def relerr(v):
    return ((v - ref) / U).astype(float)


def runmean(v):
    e = relerr(v)
    mu = np.bincount(rid, e) / cnt
    var = np.bincount(rid, e * e) / cnt - mu**2
    return mu, np.sqrt(np.maximum(var, 0.01) / cnt)


V = np.vander((xr - 1.1) / 0.4, PDEG + 1)


def chi2(target_mu, target_se, N, D):
    mu, se = runmean(res(x, N, D))
    d = target_mu - mu
    w = 1 / np.sqrt(target_se**2 + se**2)
    coef, *_ = np.linalg.lstsq(V * w[:, None], d * w, rcond=None)
    r = (d - V @ coef) * w
    return float(r @ r), r


def stage_ulps(C, xs):
    """max ulp(S_k) over the range, per k"""
    s = np.full_like(xs, C[-1])
    u = {}
    for j in range(len(C) - 2, -1, -1):
        s = s * xs + C[j]
        u[j] = np.spacing(np.abs(s)).max()
    return u


def search(N, D, tmu, tse, sweeps=2, verbose=True):
    best, _ = chi2(tmu, tse, N, D)
    xs = np.linspace(LO, HI, 2001)
    if verbose:
        print("start chi2 %.1f (runs %d)" % (best, R), flush=True)
    for sw in range(sweeps):
        for ch in "ND":
            C = N if ch == "N" else D
            us = stage_ulps(C, xs)
            for k in range(0 if ch == "N" else 1, len(C) - 1):
                if os.environ.get('ONLY') and '%s%d' % (ch, k) not in os.environ['ONLY'].split(','): continue
                uc = np.spacing(abs(C[k]))
                span = 4 * us[k] / uc
                if span < 1.5:
                    continue
                J = int(min(span, float(os.environ.get('JMAX', 20000))))
                js = np.arange(-J, J + 1)
                js = js[js != 0]
                cur = C[k]
                bj = 0
                for j in js:
                    C[k] = cur + j * uc
                    v, _ = chi2(tmu, tse, N, D)
                    if v < best - 1e-9:
                        best = v
                        bj = j
                C[k] = cur + bj * uc
                if verbose and bj:
                    print("  %s%d move %+d ulps -> chi2 %.1f" % (ch, k, bj, best), flush=True)
        if verbose:
            print(" sweep %d chi2 %.1f" % (sw, best), flush=True)
    return N, D, best


if __name__ == "__main__":
    mode = sys.argv[1]
    sweeps = int(sys.argv[2]) if len(sys.argv) > 2 else 2
    N = N5.copy()
    D = D5.copy()
    if mode == "control":
        Nt = N5.copy()
        Dt = D5.copy()
        truth = {("N", 0): 37, ("D", 1): -11, ("D", 2): 3}
        for (ch, k), j in truth.items():
            C = Nt if ch == "N" else Dt
            C[k] += j * np.spacing(abs(C[k]))
        tmu, tse = runmean(res(x, Nt, Dt))
        print("truth", truth)
    else:
        tmu, tse = runmean(b)
    N, D, best = search(N, D, tmu, tse, sweeps)
    print(
        "moves vs c5:",
        {
            ("N", k): int(round((N[k] - N5[k]) / np.spacing(abs(N5[k]))))
            for k in range(len(N))
            if N[k] != N5[k]
        },
        {
            ("D", k): int(round((D[k] - D5[k]) / np.spacing(abs(D5[k]))))
            for k in range(len(D))
            if D[k] != D5[k]
        },
    )
    if mode == "real":
        print(
            "exact: c5 far %.4f near %.4f -> far %.4f near %.4f"
            % (score(XF, BF, N5, D5), score(XN, BN, N5, D5), score(XF, BF, N, D), score(XN, BN, N, D))
        )
        open(os.path.join(HERE, "plateau_real.txt"), "w").write(
            " ".join(v.hex() for v in N) + "\n" + " ".join(v.hex() for v in D) + "\n"
        )
