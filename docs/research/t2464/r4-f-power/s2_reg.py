"""s2: per run, D regressed on separated rounding residues:
  rx  = own rounding of the stored product fl(c*LN(x)), c = a (mx <= n) or b (mx > n), entering lnF with sign +/-;
  eLx = LN(x) - ln x (Excel's LN error);
  eL  = L(r) - log1p(exact ratio)  (Excel log1p of the stored smaller ratio, incl. the ratio's rounding);
  rhL, raL, rbL = own rounding of fl(h*L), fl(a*L), fl(b*L).
Printed: coefficients (expected for 'x^c stored in log space, ln x apart, h*L stored': rx +-1, eLx (a-1) or -(b+1),
eL -h, rhL -1)."""
import sys

import mpmath
import numpy as np

import cands as C
import plib as P

LDB, st, S = C.LDB, C.st, C.S
mpf = mpmath.mpf
U = P.U


def feats(x, m, n):
    s, e = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    side1 = s["mx"] <= n
    c = a if side1 else b
    sg = 1 if side1 else -1
    Lx = s["Lx"]
    px = LDB(c) * Lx
    f = {}
    f["rx"] = sg * float((C.ex(S(px)) - C.ex(px)) * U)
    f["eLx"] = float((C.ex(Lx) - e["lx"]) * U)
    L = s["L1p2"] if side1 else s["L1p1"]
    Le = e["l1p2"] if side1 else e["l1p1"]
    f["eL"] = float((C.ex(L) - Le) * U)
    for nm, k in (("rhL", h), ("raL", a), ("rbL", b)):
        p = LDB(k) * L
        f[nm] = -float((C.ex(S(p)) - C.ex(p)) * U)
    return f, side1


def main():
    src = sys.argv[1] if len(sys.argv) > 1 else "a"
    minmax = float(sys.argv[2]) if len(sys.argv) > 2 else 20
    use = sys.argv[3].split(",") if len(sys.argv) > 3 else ["rx", "eLx", "eL", "rhL", "raL", "rbL"]
    for blk, rs, m, n in P.runs(src, minmax):
        D = np.array([P.D_units(r) for r in rs])
        fs = [feats(r["x"], m, n) for r in rs]
        side1 = fs[0][1]
        X = np.array([[f[0][k] for k in use] for f in fs])
        ok = [j for j in range(len(use)) if X[:, j].std() > 0.01]
        A = np.column_stack([X[:, ok], np.ones(len(rs))])
        cf, *_ = np.linalg.lstsq(A, D, rcond=None)
        res = D - A @ cf
        a, b = m / 2, n / 2
        print(f"{blk:20s} s{1 if side1 else 2} a-1={a-1:g} -(b+1)={-(b+1):g} h={(m+n)/2:g} | Dsd {D.std():6.1f} -> {res.std():5.2f} | "
              + " ".join(f"{use[j]}={cf[i]:+.2f}" for i, j in enumerate(ok)) + f" c={cf[-1]:+.1f}")


if __name__ == "__main__":
    main()
