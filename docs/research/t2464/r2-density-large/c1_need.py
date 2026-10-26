"""Needed shape constant per block: with E = fl(C + X) (X = loader best knobs), C exact (mpmath), the offset
dC = mean(Excel err) - mean(model err) in units of 2^-53 (absolute in E).  Compared with candidate constants:
  T1: -betaln_TOMS(a,b) [Excel GAMMALN/LN pieces, w1 toms_betaln] + k ln(k/n) + nk ln(nk/n) (exact logs)
  GL: -(GL(a)+GL(b)-GL(a+b)) + same
  LD: Loader (w6 stirlerr 'D' + LN(a+b-1) - LF0/2)
Prints the candidate's own error in the same units."""
import statistics as st
import sys
import mpmath
import dlib as L
import loader as LD
import model as M

TB = L.toms_betaln_module()

U = mpmath.mpf(2) ** 53
rows = L.load()
lim = int(sys.argv[1]) if len(sys.argv) > 1 else 200
for blk, idx in sorted(L.blocks(rows).items()):
    if blk == "old":
        continue
    sel = idx[:lim]
    a, b, k, n, nk, Cex, need, sd = L.block_need(rows, sel, lambda r: LD.xpart(r["x"], r["a"], r["b"], **M.BEST_X)[0])
    tb = -mpmath.mpf(TB.betaln(a, b)) + k * mpmath.log(k / n) + nk * mpmath.log(nk / n)
    gl = -(mpmath.mpf(L.GL(a)) + L.GL(b) - L.GL(a + b)) + k * mpmath.log(k / n) + nk * mpmath.log(nk / n)
    ks, ns, nks, Cs, lf, L5 = M.shape_consts(a, b, {})
    ld = L5 + Cs - lf / 2
    print(f"{blk:12s} ({a!r},{b!r}) need {need:+6.2f} (sd excel {sd:.2f})  T1 {float((tb - Cex) * U):+6.2f}"
          f"  GL {float((gl - Cex) * U):+6.2f}  LD {float((ld - Cex) * U):+6.2f}", flush=True)
