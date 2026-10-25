"""The shape constant: per run, the median offset (Excel - model, units) of the chain xd,ext,none,ext (exact K),
against the error (candidate - exact, units of 2^-53 absolute in ln F) of candidate forms of -lnB(a, b):
  toms   -betaln(a, b): TOMS 708 betaln with Excel LN/GAMMALN (r2-bratio Kernel, cfg gamln excel)
  tomsg  the same with TOMS's own gamln
Usage: s12_const.py SET [min max(d)] [--kside st|ln]"""
import statistics as S
import sys

import mpmath

import fblib as B
import ratlib as RL

F = B.F
LDB, st = F.LDB, F.st
mpf = mpmath.mpf
U = mpf(2) ** 53
SET = sys.argv[1]
MIN = float(sys.argv[2]) if len(sys.argv) > 2 and not sys.argv[2].startswith("--") else 0
O = dict(pw="xd", hl="ext", grp="none", est="ext", L="xl")
if "--kside" in sys.argv:
    O["kside"] = sys.argv[sys.argv.index("--kside") + 1]
rows = [r for r in B.load(SET) if max(r["d1"], r["d2"]) >= MIN]
runs: dict = {}
for r in rows:
    runs.setdefault((r["d1"], r["d2"], r["blk"]), []).append(r)
kx, kt = B.kern({"gamln": "excel"}), B.kern({"gamln": "toms"})
print("run | offset(Excel-model) | -lnB cand err: toms tomsg")
for (m, n, blk), rr in runs.items():
    a, b = m / 2, n / 2
    ex = mpmath.log(mpmath.beta(a, b))
    off = S.median(-B.F.units(RL.value(r["x"], m, n, O), r["excel"]) for r in rr)
    c_t = -(mpf(kx.betaln(a, b)) - ex) * U
    c_tg = -(mpf(kt.betaln(a, b)) - ex) * U

    print(f"{blk:24s} {off:+7.1f} | {float(c_t):+8.1f} {float(c_tg):+8.1f}")
