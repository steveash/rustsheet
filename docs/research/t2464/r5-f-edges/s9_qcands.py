"""Candidate hidden addends Q (doubles built from Excel's pieces) scored on the clean-region D(q) data: per group the
residual spread after one free C; a candidate is right on a group when the spread is within ~3 units.
Usage: s9_qcands.py [--show NAME]"""
import sys

import mpmath

import qbits as QB

L, mpf, LDB, st, ex = QB.L, QB.mpf, QB.LDB, QB.st, QB.ex
mpmath.mp.prec = 160


def cands(m, n, side):
    a, b, h = m / 2, n / 2, (m + n) / 2
    w, o = (a, b) if side == 1 else (b, a)
    c = a if side == 1 else -b
    GL = L.GL
    gh, ga, gb = GL(h), GL(a), GL(b)
    lnmn = L.LN(st(LDB(m) / LDB(n)))
    T1 = st(LDB(c) * lnmn)
    Kex = L.K_exact(m, n, side)
    Ksd = E_SD.K_of(m, n, side)
    lnB1 = st(LDB(st(LDB(ga) + LDB(gb))) - LDB(gh))
    lnB2 = st(LDB(st(LDB(gh) - LDB(ga))) - LDB(gb))  # -lnB
    A = st(LDB(h - 0.5) * L.L_xl(st(LDB(w) / LDB(o))))
    lnw2pi = st(LDB(0.5) * L.LN(st(LDB(w) / LDB(2 * mpmath.pi))))
    stl = [float(QB.L.F.stirlerr_exact(v)) for v in (h, a, b)]
    ST = st(LDB(st(LDB(stl[0]) - LDB(stl[1]))) - LDB(stl[2]))
    out = {
        "0": mpf(0),
        "fl(K')": mpf(float(Kex)),
        "fl(Ksd)": ex(LDB(float(ex(Ksd)))),
        "T1=fl(c LN(m/n))": ex(T1),
        "-lnB1": -ex(lnB1),
        "lnB2": ex(lnB2),
        "fl(lnB2+T1)": ex(st(LDB(lnB2) + LDB(T1))),
        "fl(T1-lnB1)": ex(st(LDB(T1) - LDB(lnB1))),
        "A": ex(A),
        "fl(A+lnw2pi)": ex(st(LDB(A) + LDB(lnw2pi))),
        "fl(fl(A+lnw2pi)+ST)": ex(st(LDB(st(LDB(A) + LDB(lnw2pi))) + LDB(ST))),
        "fl(A+ST)": ex(st(LDB(A) + LDB(ST))),
        "ST": ex(LDB(ST)),
        "lnw2pi": ex(LDB(lnw2pi)),
        "fl(ST+lnw2pi)": ex(st(LDB(ST) + LDB(lnw2pi))),
        "-gb": -mpf(gb), "-ga": -mpf(ga), "gh": mpf(gh),
        "fl(gh-ga)": ex(st(LDB(gh) - LDB(ga))),
        "fl(T1+gh)": ex(st(LDB(T1) + LDB(gh))),
    }
    return out


import edgemodel as _M  # noqa: E402

E_SD = _M.SD
FORM = next((a.split("=")[1] for a in sys.argv if a.startswith("form=")), "x")
G = QB.groups(form=FORM)
tot: dict = {}
for k, cs in sorted(G.items()):
    if len({float(c["q"]) for c in cs}) < 2:
        continue
    cd = cands(*k)
    for name, Q in cd.items():
        sp = QB.spread(cs, Q)
        t = tot.setdefault(name, [0, 0, []])
        t[0] += sp <= 3.5
        t[1] += 1
        t[2].append(f"({k[0]:g},{k[1]:g})s{k[2]}:{sp:.0f}")
for name, (ok, nn, det) in sorted(tot.items(), key=lambda kv: -kv[1][0]):
    print(f"{name:22s} {ok:3d}/{nn}  " + (" ".join(det) if "--show" in sys.argv and name in sys.argv else ""))
