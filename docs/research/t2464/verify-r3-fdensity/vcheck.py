"""Checks on the worker's candidate and claims:
 1. an independent re-implementation of ratio-xl-e from the RESULTS description agrees with c1_model.value;
 2. the Excel primitives it relies on (r3lib LN, EXP V6) agree with the engine harness (excelmath) on doubles;
 3. F.DIST(1,m,n,FALSE) == F.DIST(1,n,m,FALSE) on the sw sweep (swapped pairs, bit for bit);
 4. hold-out hygiene: x values shared between the worker's hold-out and its in-sample probe a.
usage: vcheck.py"""
import collections
import random
import sys

import mpmath
import numpy as np

import vlib as V

M1 = V.model("ratio-xl-e")  # import the worker stack first (its own sys.path order)
sys.path += [str(V.T), str(V.T / "r3-loader-bd0"), str(V.T / "r2-log1p")]
import excelmath  # noqa: E402
import excel_log1p as XL  # noqa: E402
import r3lib as R3  # noqa: E402

LDB = np.longdouble
mpf = mpmath.mpf


def own_ratio_xl_e(x, m, n):
    """From the description: mx = fl(m x); mx <= n: K + a ln(m/n) + fl((a-1) LN x) - h L(fl(mx/n));
    else K - b ln(m/n) + fl(-(b+1) LN x) - h L(fl(n/mx)); L = series (r <= 0.375) or LN(fl(1+r)); E extended."""
    a, b, h = mpf(m) / 2, mpf(n) / 2, (mpf(m) + mpf(n)) / 2
    K = mpmath.loggamma(h) - mpmath.loggamma(a) - mpmath.loggamma(b)
    mx = float(LDB(m) * LDB(x))
    if mx <= n:
        r, c, k2 = float(LDB(mx) / LDB(n)), m / 2 - 1, a * mpmath.log(mpf(m) / n)
    else:
        r, c, k2 = float(LDB(n) / LDB(mx)), -(n / 2 + 1), -b * mpmath.log(mpf(m) / n)
    L = XL.log1p_series(r) if r <= 0.375 else R3.LN(float(LDB(1) + LDB(r)))
    tx = float(LDB(c) * LDB(R3.LN(x)))
    E = K + k2 + mpf(tx) - h * mpf(L)
    return float(R3.exp_v6(R3._ld_of(E), True))


def main():
    m1 = M1
    rows = []
    for cid in ("t2513-r3-fdensity-hold", "t2513-verify-r3-fdensity-hold", "t2513-r3-fdensity-holdsw"):
        try:
            rows += V.case_rows(cid)
        except FileNotFoundError:
            pass
    diff = sum(own_ratio_xl_e(r["x"], r["d1"], r["d2"]) != m1(r["x"], r["d1"], r["d2"]) for r in rows)
    print(f"1. own ratio-xl-e vs c1_model.value: {diff} differ of {len(rows)}")

    rng = random.Random(7)
    xs = [rng.uniform(0.01, 300) for _ in range(3000)]
    ln_h = excelmath.excel("LN", xs)
    print("2. r3lib LN vs harness LN:", sum(R3.LN(x) != y for x, y in zip(xs, ln_h)), "differ of", len(xs))
    es = [rng.uniform(-40, 5) for _ in range(3000)]
    ex_h = excelmath.excel("EXP", es)
    print("   r3lib exp_v6 (double E) vs harness EXP:",
          sum(float(R3.exp_v6(LDB(e), True)) != y for e, y in zip(es, ex_h)), "differ of", len(es))

    sw = V.case_rows("t2513-r3-fdensity-sw")
    at1 = {(r["d1"], r["d2"]): r["excel"] for r in sw if r["x"] == 1.0}
    pairs = [(k, (k[1], k[0])) for k in at1 if k[0] < k[1] and (k[1], k[0]) in at1]
    eq = sum(at1[p] == at1[q] for p, q in pairs)
    print(f"3. x = 1 swapped pairs equal: {eq} of {len(pairs)}")
    at275 = {(r["d1"], r["d2"]): r["excel"] for r in sw if r["x"] == 2.75}
    print("   (no swap symmetry expected at x = 2.75; distinct pairs there:", len(at275), ")")

    a = {r["x"] for r in V.case_rows("t2513-r3-fdensity-a")}
    hx = [r["x"] for r in V.case_rows("t2513-r3-fdensity-hold")]
    print(f"4. worker hold-out x values also used in probe a (other shapes): {sum(x in a for x in hx)} of {len(hx)}")
    vh = [r["x"] for r in rows if r["case"] == "t2513-verify-r3-fdensity-hold"]
    print(f"   verifier hold-out x values used anywhere else: {sum(x in a or x in set(hx) for x in vh)} of {len(vh)}")
    cnt = collections.Counter((r["case"], r["x"], r["d1"], r["d2"]) for r in rows)
    print("   duplicate (x, d1, d2) within hold-out cases:", sum(v - 1 for v in cnt.values()))


if __name__ == "__main__":
    main()
