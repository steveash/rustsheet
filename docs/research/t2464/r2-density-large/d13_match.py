"""Matched filter: correlate Excel's density error along a run with the exact rounding error (fl53(Q) - Q, relative to
ulp) of many candidate intermediates Q(x) (TOMS brcomp, Loader, log-form, power-form atoms), per block.
Prints candidates with |rho| >= 0.12 (noise ~ 1/sqrt(n))."""
import math
import sys
import numpy as np
import mpmath
import dlib as L

mpmath.mp.prec = 120
rows = L.load()


def rlog1(e):
    return e - mpmath.log1p(e)


def atoms(x, a, b):
    X, A, B = mpmath.mpf(x), mpmath.mpf(a), mpmath.mpf(b)
    Y = 1 - X
    Yd = mpmath.mpf(1.0 - x)
    out = {}
    for tag, p, q in (("ab", A, B), ("a1b1", A - 1, B - 1)):
        s = p + q
        lam = p - s * X
        out.update({
            f"{tag}:p lnx": p * mpmath.log(X), f"{tag}:q lny": q * mpmath.log(Y),
            f"{tag}:p lnx+q lny": p * mpmath.log(X) + q * mpmath.log(Y),
            f"{tag}:s x": s * X, f"{tag}:s y": s * Yd, f"{tag}:lambda": lam,
            f"{tag}:-lam/p": -lam / p, f"{tag}:lam/q": lam / q,
            f"{tag}:rlog1(-lam/p)": rlog1(-lam / p), f"{tag}:rlog1(lam/q)": rlog1(lam / q),
            f"{tag}:p rlog1": p * rlog1(-lam / p), f"{tag}:q rlog1": q * rlog1(lam / q),
            f"{tag}:p u + q v": p * rlog1(-lam / p) + q * rlog1(lam / q),
            f"{tag}:p/(s x)": p / (s * X), f"{tag}:q/(s y)": q / (s * Yd),
            f"{tag}:x^p": X ** p, f"{tag}:y^q": Y ** q, f"{tag}:x^p y^q": X ** p * Y ** q,
            f"{tag}:s x/p": s * X / p, f"{tag}:s y/q": s * Yd / q,
        })
    out.update({"x*y": X * Yd, "lnx": mpmath.log(X), "lny": mpmath.log(Y), "x/y": X / Yd, "y/x": Yd / X,
                "pdf": X ** (A - 1) * Y ** (B - 1) / mpmath.beta(A, B), "ln pdf": mpmath.log(X ** (A - 1) * Y ** (B - 1) / mpmath.beta(A, B)),
                "brcomp": X ** A * Y ** B / mpmath.beta(A, B), "brcomp/x": X ** (A - 1) * Y ** B / mpmath.beta(A, B),
                "brcomp/y": X ** A * Y ** (B - 1) / mpmath.beta(A, B)})
    return out


for blk in sys.argv[1:]:
    idx = sorted(L.blocks(rows)[blk], key=lambda i: rows[i]["x"])
    e = np.array([L.err_units(rows[i]["excel"], rows[i]["exact"]) for i in idx])
    e -= e.mean()
    feats = {}
    for i in idx:
        r = rows[i]
        for name, v in atoms(r["x"], r["a"], r["b"]).items():
            fv = float(v)
            u = math.ulp(fv) if fv != 0 else 1.0
            feats.setdefault(name, []).append(float((mpmath.mpf(L.R53(v)) - v) / u))
    res = []
    for name, f in feats.items():
        f = np.array(f)
        if f.std() < 1e-6:
            continue
        rho = np.corrcoef(f, e)[0, 1]
        res.append((abs(rho), rho, name))
    res.sort(reverse=True)
    r0 = rows[idx[0]]
    print(f"{blk} ({r0['a']:g},{r0['b']:g}) x~{r0['x']:.4f}: " + "; ".join(f"{n} {r:+.2f}" for _, r, n in res[:8]))
