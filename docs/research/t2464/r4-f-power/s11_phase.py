"""s11: hidden-constant phase scan.  Hypothesis: Excel stores T = fl(C + v) for an unknown constant C, with v an
x-dependent piece of the m1 'xl' model.  For each grid ulp 2^k units and phase phi (C/ulp mod 1), the residue
rho = fl(C + v) - (C + v) is regressed against the m1 residual (Excel - model); best (k, phi) per run by
residual sd.  v in: hL = h*L(r); P = sg*fl(c*LN(x)); PL = P - h*L; PLX = P - LN(x) - h*L.
Usage: s11_phase.py SRC MINMAX"""
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P

LDB, S, ex = C.LDB, C.S, C.ex
U = P.U


def vparts(x, m, n):
    s, _ = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    side1 = s["mx"] <= n
    c, sg = (a, 1) if side1 else (b, -1)
    Pp = sg * S(LDB(c) * s["Lx"])
    H = LDB(h) * M.L_form(s, side1, "xl", n)
    return {"hL": -float(ex(H) * U), "P": float(ex(Pp) * U), "PL": float(ex(Pp - H) * U),
            "PLX": float(ex(Pp - H - s["Lx"]) * U)}


def scan(res, v, ks=range(3, 10), nph=256):
    best = (res.std(), None, None, None)
    for k in ks:
        u = 2.0 ** k
        for j in range(nph):
            ph = j / nph
            t = v / u + ph
            rho = (np.round(t) - t) * u
            A = np.column_stack([rho, np.ones(len(res))])
            cf, *_ = np.linalg.lstsq(A, res, rcond=None)
            sd = (res - A @ cf).std()
            if sd < best[0]:
                best = (sd, k, ph, cf[0])
    return best


def main():
    src, minmax = sys.argv[1], float(sys.argv[2])
    for blk, rs, m, n in P.runs(src, minmax):
        D = np.array([P.D_units(r) for r in rs])
        Dm = np.array([float((M.lnF_model(r["x"], m, n, "xl") - mpmath.log(r["exact"])) * U) for r in rs])
        res = D - Dm
        V = [vparts(r["x"], m, n) for r in rs]
        out = []
        for key in ("hL", "P", "PL", "PLX"):
            v = np.array([d[key] for d in V])
            v = v - v[0]  # phase absorbs the offset; keep magnitudes small for float precision
            sd, k, ph, cf = scan(res, v)
            out.append(f"{key}: {sd:5.1f} k={k} ph={ph} c={cf if cf is None else round(cf, 2)}")
        print(f"{blk:20s} res {res.std():6.1f} | " + " | ".join(out))


if __name__ == "__main__":
    main()
