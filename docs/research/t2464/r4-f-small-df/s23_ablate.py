"""Ablations of the frozen M-sd constant on in-sample sets: 'full'; 'no-e' (e(v) replaced by its linear gauge, so
only C0 + the side term remain); 'no-side' (side term dropped); 'C0-only'. Usage: s23_ablate.py SET [SET ...]"""
import sys

import sdmodel as SD
import slib as L

orig_e = SD.e_of
lin = lambda v: float(SD._ALPHA * v + SD._GAMMA)  # noqa: E731
LDB, st, mp = L.LDB, L.st, L.mpmath


def make(use_e, use_side):
    def delta(m, n, side):
        a, b, h = m / 2, n / 2, (m + n) / 2
        e = orig_e if use_e else (lambda v: 0.0 if v in (0.5, 1.0) else lin(v))
        d = e(h) - e(a) - e(b) + SD.C0
        if use_side:
            w, o = (a, b) if side == 1 else (b, a)
            p, q = (LDB(h - 0.5) * L.L_xl(st(LDB(w) / LDB(o)))).as_integer_ratio()
            d += float((L.mpf(p) / q - (h - 0.5) * mp.log(L.mpf(h) / o)) / L.U)
        return d
    return delta


for s in sys.argv[1:]:
    rows = L.load(s)
    for nm, (ue, us) in {"full": (1, 1), "no-e": (0, 1), "no-side": (1, 0), "C0-only": (0, 0)}.items():
        dl = make(ue, us)
        K = {}

        def fn(r):
            side, _ = L.side_r(r["x"], r["d1"], r["d2"])
            k = (r["d1"], r["d2"], side)
            if k not in K:
                K[k] = L.ld(L.K_exact(*k)) + LDB(dl(*k)) * LDB(2.0) ** -53
            return L.mxd_K(r["x"], r["d1"], r["d2"], K[k])
        print(s, f"{nm:8s}", L.F.fmt(L.score_rows(rows, fn)["all"]))
