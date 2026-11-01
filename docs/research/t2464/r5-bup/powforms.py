"""T-3568 r5-bup: do bup's terms take x^i through an exponent? sd of eS - eModel (BETA.DIST, D bgrat, P > 0).

eS as in sreg.py. Each model is S with term i = R_i X_i (R_i the exact product of TOMS's stored ratios) and X_i:
  exact    y0^i exactly
  explnd   exp(fl(i * fl(ln y0))) with exp exact: the power through a double exponent
  explnx   exp(fl(fl(ln y0) * i)) where ln y0 is correctly rounded but the product in extended (one rounding at the end)
  wholed   the whole term exp(fl(sum of ln stored ratios + i ln y0)): an exponent-space recurrence in doubles
  cumd     cumulative exponent: E_i = fl(E_{i-1} + fl(ln(ratio_i * y0))), term exp(E_i)
usage: powforms.py tune|hold"""
from __future__ import annotations

import statistics
import sys
from collections import defaultdict

import mpmath

import score as s

M = mpmath.mpf
U = M(2) ** -53


def rnd(v) -> M:
    return M(float(v))


def forms(c, a0, y0, n):
    apb, ap1 = c + a0, c + 1.0
    ly = rnd(mpmath.log(y0))
    out = defaultdict(lambda: M(0))
    r = M(1)
    lsum = M(0)
    e_cum = M(0)
    for i in range(n):
        if i:
            ratio = (M(apb) + i - 1) / (M(ap1) + i - 1)
            r *= ratio
            lsum = rnd(lsum + rnd(mpmath.log(ratio)))
            e_cum = rnd(e_cum + rnd(mpmath.log(ratio * y0)))
        out["exact"] += r * M(y0) ** i
        out["explnd"] += r * mpmath.exp(rnd(i * ly))
        out["wholed"] += mpmath.exp(rnd(lsum + rnd(i * ly)))
        out["cumd"] += mpmath.exp(e_cum)
    return out


g = defaultdict(lambda: defaultdict(list))
for r in s.load(sys.argv[1]):
    if r["P"] == 0 or r["region"] != "beta" or r["dbr"] != "bgrat":
        continue
    c, a0, n = r["c"], r["a0"], r["n"]
    f = forms(c, a0, r["y0"], n)
    pre = M(float(s.ret_of(r, "x87")))
    se = f["exact"]
    es = float(((M(r["excel"]) - r["D"]) / (pre * se) - 1) / U)
    nb = "n<=9" if n <= 9 else "n>=10"
    for k, v in f.items():
        em = float((v / se - 1) / U)
        g[nb][k].append(es - em)
        g["all"][k].append(es - em)
print("| X_i form | " + " | ".join(sorted(g)) + " |")
print("|---|" + "---|" * len(g))
for k in g["all"]:
    print(f"| {k} | " + " | ".join(f"sd {statistics.pstdev(g[b][k]):.1f}, median {statistics.median(g[b][k]):.1f}" for b in sorted(g)) + " |")
