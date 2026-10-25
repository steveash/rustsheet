"""Candidate shape constants C (E = C + X) against the needed offset (c1_need.py method, fixed-shape blocks only).
Candidates (error vs exact C in units of 2^-53):
  LDt  : Loader, stirlerr table at half-integers (w6 Arith D), LN(a+b-1) - LF0/2       (= c1_need LD)
  LDg  : Loader, stirlerr by formula everywhere: GL(v+1) - (v+.5) LN(v) + v - ln sqrt(2pi), x87 stored
  LDg1 : same, LF1
  LDe  : stirlerr exact (mpmath), LF0 with Excel LN
  T1   : -TOMS betaln (Excel GL/LN) + k ln(k/n) + nk ln(nk/n)"""
import statistics as st
import sys
import mpmath
import dlib as L
import loader as LD
import model as M

TB = L.toms_betaln_module()

U = mpmath.mpf(2) ** 53
rows = L.load()
SQ = mpmath.mpf(M.H.M_LN_SQRT_2PI)


def stir_g(v):
    v = mpmath.mpf(v)
    t = L.D(mpmath.mpf(L.D(v + 0.5)) * mpmath.mpf(L.LN(float(v))))
    return mpmath.mpf(L.D(mpmath.mpf(L.D(mpmath.mpf(L.D(mpmath.mpf(L.GL(float(v + 1))) - t)) + v)) - SQ))


lim = 300


def main():
    print("block (a,b): need | LDt LDg LDg1 LDe T1")
    for blk, idx in sorted(L.blocks(rows).items()):
        if blk == "old" or blk.startswith("absweep"):
            continue
        sel = idx[:lim]
        a, b, k, n, nk, Cex, need, _ = L.block_need(rows, sel, lambda r: LD.xpart(r["x"], r["a"], r["b"], **M.BEST_X)[0])
        ks, ns, nks, Cs, lf0, L5 = M.shape_consts(a, b, {})
        lf1 = M.lf_value(ks, ns, "LF1")
        Csg = stir_g(ns) - stir_g(ks) - stir_g(nks)
        ste = lambda v: mpmath.loggamma(mpmath.mpf(v) + 1) - (mpmath.mpf(v) + 0.5) * mpmath.log(v) + v - mpmath.log(mpmath.sqrt(2 * mpmath.pi))  # noqa: E731,E501
        Cse = ste(ns) - ste(ks) - ste(nks)
        cands = [L5 + Cs - lf0 / 2, L5 + Csg - lf0 / 2, L5 + Csg - lf1 / 2, L5 + Cse - lf0 / 2,
                 -mpmath.mpf(TB.betaln(a, b)) + k * mpmath.log(k / n) + nk * mpmath.log(nk / n)]
        print(f"{blk:14s} ({a:g},{b:g}) need {need:+6.2f} | " + " ".join(f"{float((c - Cex) * U):+7.2f}" for c in cands),
              flush=True)


if __name__ == "__main__":
    main()
