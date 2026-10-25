"""Constant model: delta(m, n, side) = e(h) - e(a) - e(b) + c0 + R_side, with r3-loader-const's e(v) (grid_e.tsv)
for v >= 1.5 and free e(0.5), e(1), c0 (least squares), and R_side one of several deterministic side-piece rounding
errors (units, computed - exact). Observed delta from s4_kscan outputs (M-xd base, groups with best >= MINC).
--plant R: positive control, the observed offsets are replaced by synthetic ones from side form R, c0 = 3,
e(.5) = e(1) = 0 and N(0, 0.3) noise (the fit must pick R).
Usage: s19_sidefit.py MINC file [file ...] [--show R] [--plant R]"""
import re
import sys

import numpy as np

import slib as L

LDB, st, mp = L.LDB, L.st, L.mpmath
show = sys.argv[sys.argv.index("--show") + 1] if "--show" in sys.argv else None
files = [a for a in sys.argv[2:] if a.endswith(".out")]
MINC = int(sys.argv[1])
E = {float(v): float(e) for v, e in (ln.split("\t") for ln in (L.T / "r3-loader-const" / "grid_e.tsv").read_text().splitlines()[1:])}


def fr(v):
    p, q = LDB(v).as_integer_ratio()
    return L.mpf(p) / q


def u(v):
    return float(v / L.U)


def R_forms(m, n, s):
    a, b, h = m / 2, n / 2, (m + n) / 2
    c = a if s == 1 else -b
    o, w = (b, a) if s == 1 else (a, b)  # K_s = (h-1/2) ln(h/o) + 1/2 ln(w/2pi) + S
    t = L.mpf(m) / n
    out = {"none": 0.0}
    out["fl(c LN(fl(m/n)))"] = u(L.mpf(st(LDB(c) * L.LN(st(LDB(m) / LDB(n))))) - c * mp.log(t))
    out["c LN(fl(m/n)) ext"] = u(fr(LDB(c) * L.LN(st(LDB(m) / LDB(n)))) - c * mp.log(t))
    hr = st(LDB(h) / LDB(o))
    ex1 = (h - 0.5) * mp.log(L.mpf(h) / o)
    out["fl((h-.5)LN(h/o))"] = u(L.mpf(st(LDB(h - 0.5) * L.LN(hr))) - ex1)
    out["(h-.5)LN(h/o) ext"] = u(fr(LDB(h - 0.5) * L.LN(hr)) - ex1)
    wr = st(LDB(w) / LDB(o))
    out["fl((h-.5)L(w/o))"] = u(L.mpf(st(LDB(h - 0.5) * L.L_xl(wr))) - ex1)
    out["(h-.5)L(w/o) ext"] = u(fr(LDB(h - 0.5) * L.L_xl(wr)) - ex1)
    ex2 = 0.5 * mp.log(L.mpf(w))
    out["+fl(.5 LN w)"] = out["fl((h-.5)LN(h/o))"] + u(L.mpf(st(LDB(0.5) * L.LN(w))) - ex2)
    return out


obs = []
for f in files:
    for line in open(L.HERE / f):
        mm = re.match(r"\((\d+),(\d+)\) s(\d) n\s+(\d+) best\s+(\d+) delta \[\s*([-+.\d]+),\s*([-+.\d]+)\]", line)
        if mm and int(mm[5]) >= MINC:
            m, n = float(mm[1]), float(mm[2])
            a, b = m / 2, n / 2
            if a + b in E:
                obs.append((m, n, int(mm[3]), 0.5 * (float(mm[6]) + float(mm[7]))))
names = list(R_forms(1.0, 2.0, 1))
if "--plant" in sys.argv:
    pl = sys.argv[sys.argv.index("--plant") + 1]
    rng = np.random.default_rng(11)
    obs = [(m, n, s, sum(sg * (E[v] if v >= 1.5 else 0.0) for sg, v in ((1, (m + n) / 2), (-1, m / 2), (-1, n / 2)))
            + 3.0 + R_forms(m, n, s)[pl] + rng.normal(0, 0.3)) for m, n, s, _ in obs]
    print(f"# CONTROL: planted {pl}")
for nm in names:
    Y, X = [], []
    for m, n, s, d in obs:
        a, b, h = m / 2, n / 2, (m + n) / 2
        known = sum(sg * (E[v] if v >= 1.5 else 0.0) for sg, v in ((1, h), (-1, a), (-1, b)))
        cnt = [sum(-1 for v in (a, b) if v == 0.5) + (1 if h == 0.5 else 0),
               sum(-1 for v in (a, b) if v == 1.0) + (1 if h == 1.0 else 0)]
        Y.append(d - known - R_forms(m, n, s)[nm])
        X.append([1.0] + cnt)
    X, Y = np.array(X), np.array(Y)
    coef, *_ = np.linalg.lstsq(X, Y, rcond=None)
    res = Y - X @ coef
    print(f"{nm:22s} RMS {np.sqrt((res**2).mean()):.2f} within1 {(abs(res) <= 1).sum():2d}/{len(res)} "
          f"c0 {coef[0]:+.2f} e(.5) {coef[1]:+.2f} e(1) {coef[2]:+.2f}")
    if nm == show:
        for (m, n, s, d), rr in zip(obs, res):
            print(f"   ({m:g},{n:g}) s{s} obs {d:+6.2f} resid {rr:+6.2f}")
