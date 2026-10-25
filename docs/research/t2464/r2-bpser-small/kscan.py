"""Product-form test for bpser's prefactor (TOMS b0 <= 1 style: x^a computed on its own, times a shape constant K):
I = fl(fl(X*K) * s) or fl(X * fl(K*s)), X = EXP(fl(a LN x)) or EXP(extended a LN x), s = w3's series; scan K over
K0 (1 + k 2^-53), K0 = 1/B(a,b) correctly rounded, and report the best k and its exact count per run (or per shape).
usage: kscan.py [sets=runs] [min=50] [kmax=80]"""
import math
import sys
from collections import defaultdict

import mpmath

import lib
from lib import mpf, sx

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
sets = tuple(kw.get("sets", "runs").split(","))
kmax = int(kw.get("kmax", "80"))
rows = lib.load_probe(tuple(s for s in sets if s in lib.PROBES))
res = lib.run([(r["a"], r["b"], r["x"], r["y"]) for r in rows], lib.V())
shapes = defaultdict(list)
for r, q in zip(rows, res):
    if q[2] == "w_bpser" and r["side"] == "lower":
        fn, a, b, x = q[3][-1]
        shapes[(a, b)].append((x, r["excel"]))
for (a, b), cells in sorted(shapes.items()):
    if len(cells) < int(kw.get("min", "50")):
        continue
    with mpmath.workdps(40):
        K0 = float(1 / mpmath.beta(mpf(a), mpf(b)))
    pre = []
    for x, e in cells:
        s = lib.Model.series_w3(a, b, x)
        l = lib.LN(x)
        pre.append((lib.EXP(sx("*", a, l)), lib.EXPX(lib.fl64(mpf(a) * mpf(l))), s, e))
    out = []
    for xi, xname in ((0, "X=EXP(fl(a ln x))"), (1, "X=EXP(ext a ln x)")):
        for form in ("(X*K)*s", "X*(K*s)"):
            best = (-1, None)
            for k in range(-kmax, kmax + 1):
                K = K0 * (1 + k * 2.0**-53)
                if form == "(X*K)*s":
                    n = sum(sx("*", sx("*", p[xi], K), p[2]) == p[3] for p in pre)
                else:
                    n = sum(sx("*", p[xi], sx("*", K, p[2])) == p[3] for p in pre)
                if n > best[0]:
                    best = (n, k)
            out.append(f"{xname} {form}: best k={best[1]:+d} {best[0]}/{len(pre)}")
    print(f"{lib.sub_branch('bpser', a, b)} a={a} b={b}: " + " | ".join(out))
