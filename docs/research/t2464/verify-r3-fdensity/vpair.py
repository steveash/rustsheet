"""Paired comparison of two variants (or CR) on a set: cells where only one is within 15 digits / within 4 ulp /
bit-exact, with a two-sided sign-test p value on the discordant cells.
usage: vpair.py SET VARIANT_A VARIANT_B"""
import sys

import mpmath

import vlib as V
import vscore as S


def main():
    s, na, nb = sys.argv[1:4]
    rows, _ = S.rows_of(s)
    fa, fb = (None if n == "CR" else V.model(n) for n in (na, nb))
    disc = [[0, 0] for _ in range(3)]
    for r in rows:
        cr = float(V.f_exact(r["x"], r["d1"], r["d2"]))
        va = cr if fa is None else fa(r["x"], r["d1"], r["d2"])
        vb = cr if fb is None else fb(r["x"], r["d1"], r["d2"])
        ca, cb = V.classify(va, r["excel"]), V.classify(vb, r["excel"])
        for k in range(3):
            disc[k][0] += ca[k] and not cb[k]
            disc[k][1] += cb[k] and not ca[k]
    for lab, (a, b) in zip(("exact", "within4", "w15"), disc):
        n = a + b
        p = float(min(1, 2 * sum(mpmath.binomial(n, i) for i in range(min(a, b) + 1)) / mpmath.mpf(2) ** n)) if n else 1
        print(f"{s} {na} vs {nb} {lab:7s}: +{a} / -{b}  sign-test p {p:.2g}")
    V.save()


if __name__ == "__main__":
    main()
