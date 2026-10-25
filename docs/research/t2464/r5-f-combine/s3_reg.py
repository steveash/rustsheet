"""s3: local regression per r > 0.375 run.  Target: the centre of Excel's admissible E interval, minus the exact
E* = ln(F x), in units.  Regressors are the rounding errors of Excel's pieces, each scaled so that the M-sd/ext
structure (E = K + P - h LN(fl(1+r)), all extended) predicts coefficient 1:
  xP   sg*c*eLN(x)          (LN x error through the stored product)
  pP   sg*(fl(c LN x) - c LN x)   (the product's own rounding)
  rr   -h * (fl(r) - r)/(1+r)    (r's rounding)
  ss   -h * (fl(1+r) - (1+fl(r)))/s   (s's rounding)
  ls   -h * eLN(s)          (Excel's LN error at s)
Printed per run: coefficients (and intercept), residual sd, interval width.  Usage: s3_reg.py SET"""

import sys

import mpmath
import numpy as np

import clib as C

U, mpf = C.U, C.mpf
NAMES = ("xP", "pP", "rr", "ss", "ls")


def regressors(r):
    x, m, n = r["x"], r["d1"], r["d2"]
    pc = C.pieces(x, m, n)
    X, side = mpf(x), pc["side"]
    c, sg = (pc["a"], 1) if side == 1 else (pc["b"], -1)
    h = pc["h"]
    Lx = C.ex(pc["Lx"])
    eLx = (Lx - mpmath.log(X)) * U
    pP = (C.ex(sg * pc["P"]) - c * Lx) * U * sg
    rex = m * X / n if side == 1 else n / (m * X)
    rr = C.ex(pc["r"])
    s = C.st(C.LDB(1) + C.LDB(pc["r"]))
    es = (C.ex(s) - 1 - rr) * U
    LNs = C.ex(C.SL.LN(s))
    els = (LNs - mpmath.log(C.ex(s))) * U
    return [
        float(sg * c * eLx),
        float(pP),
        float(-h * (rr - rex) * U / (1 + rex)),
        float(-h * es / C.ex(s)),
        float(-h * els),
    ]


def main():
    name = sys.argv[1]
    allc = []
    for key, rs in sorted(C.runs(C.load(name)).items()):
        m, n, side, _ = key
        Y, Xs, W = [], [], []
        for r in rs:
            iv = C.e_interval(r)
            if iv is None:
                continue
            Y.append((iv[0] + iv[1]) / 2)
            W.append(iv[1] - iv[0])
            Xs.append(regressors(r))
        if len(Y) < 12:
            continue
        A = np.column_stack([np.ones(len(Y)), np.array(Xs)])
        coef, *_ = np.linalg.lstsq(A, np.array(Y), rcond=None)
        res = np.array(Y) - A @ coef
        allc.append(coef[1:])
        fx = np.array(Y) - np.array(Xs).sum(axis=1)
        fx_sd = float(fx.std())
        print(
            f"({m:g},{n:g}) s{side} x{rs[0]['x']:.4g} | "
            + " ".join(f"{k} {v:+5.2f}" for k, v in zip(NAMES, coef[1:]))
            + f" | c0 {coef[0]:+6.1f} sd {res.std():5.2f} w {np.mean(W):4.2f}"
        )
    allc = np.array(allc)
    print("median " + " ".join(f"{k} {v:+5.2f}" for k, v in zip(NAMES, np.median(allc, axis=0))))
    C.save()


if __name__ == "__main__":
    main()
