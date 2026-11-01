"""T-2829: Excel's ln B(a, b) inside F.TEST's bpser, read off probes/t2829-ftest-ln-beta-grid and
probes/t2829-ftest-a306-shapes, and scored against candidate forms.

Usage: python ln_beta_map.py (needs mpmath and numpy on x86-64, as
docs/research/t2464/r2-bratio/bratio_excel.py does; run from any directory).

Method: with x near 1E-12 F.TEST takes bpser, p = 2 EXP(z) s with z = fl(a LN x) - L held unstored
(statistics.md "F.TEST / FTEST"), so Excel's L = fl(a LN x) - ln(p / (2 s)), to about 1E-16 absolute.
x is the two-pass VAR.S ratio, s the stored-x87 series of `f_test_tail.rs`. The script prints
k = (L_excel - fl(L_accurate)) / ulp(L) per cell: every k is within 0.2 of an integer, so Excel's L is
a stored double (the same test on L + ln a, L + ln b, L + lnGamma(a + b) gives random fractions),
then how many cells each candidate form reproduces.
"""

import json
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent.parent
sys.path[:0] = [str(HERE.parent / "t2464" / "r2-bratio"), str(HERE.parent / "t2464")]
import bratio_excel as B  # noqa: E402 - needs the sys.path entries set above
import mpmath  # noqa: E402
from mpmath import mpf  # noqa: E402

mpmath.mp.prec = 200
CASES = ROOT / "eval" / "suites" / "probes" / "cases"
SMALL = [1.005, 0.5, 1, 1, 0.2, 0.1, 100, -1, -2.5, 0.5, 3.14159, -0.0001, -0.0001, 2, 0.7, -3, 1.5]
LARGE = [1, -0.0001, 123456789.123, 0, 1, 0.3, 7, 100, 0, 1.005, 1e-15, -0.0001, 3.14159, 1.005, 3.14159, 0.5, 1,
         0.2, 0.1, 3.14159, 100, 1, -2.5, 0.5, 2, 0.7, -3, 1.5, 4, 0.25, -1, 9, 0.6, 2.2, 1.1, -0.4, 5, 0.3, 0.8, 1.9]
A = [2.5, 3, 3.5, 4, 4.5, 5, 6, 7, 8]
BS = [8.5, 9, 9.5, 10, 10.5, 11, 11.5, 12, 13, 15.5, 19.5]


def var_s(v: list[float]) -> float:
    m = sum(v) / len(v)
    return sum((t - m) ** 2 for t in v) / (len(v) - 1)


def series(a: float, b: float, x: float) -> float:
    sx = B.sx
    s, c, n = sx("/", 1.0, a), 1.0, 0.0
    while True:
        n += 1.0
        c = sx("*", sx("*", c, sx("+", sx("-", 0.5, sx("/", b, n)), 0.5)), x)
        s2 = sx("+", s, sx("/", c, sx("+", a, n)))
        if s2 == s:
            return s
        s = s2


def accurate(a: float, b: float):
    return mpmath.log(mpmath.beta(mpf(a), mpf(b)))


def excel_k(a: float, b: float, x: float, p: float) -> float:
    big_l = mpf(B.sx("*", a, B.LN(x))) - mpmath.log(mpf(p) / (2 * mpf(series(a, b, x))))
    acc = float(accurate(a, b))
    return float((big_l - mpf(acc)) / math.ulp(acc))


def grid() -> list[tuple[float, float, float]]:
    oracle = json.loads((CASES / "t2829-ftest-ln-beta-grid.oracle.json").read_text())["sheets"]["S"]
    rows, r = [], 1
    for a in A:
        for b in BS:
            small, large = SMALL[: int(2 * a + 1)], LARGE[: int(2 * b + 1)]
            ds, db = len(small) - 1, len(large) - 1
            x = ds / (ds + db * (var_s(large) / var_s(small)))
            rows.append((a, b, excel_k(a, b, x, oracle[f"A{r}"]["v"])))
            r += 1
    return rows


def kk(big_l: float, a: float, b: float) -> int:
    acc = float(accurate(a, b))
    return round((big_l - acc) / math.ulp(acc))


def candidates() -> dict:
    k = B.Kernel(B.BEST)
    gl = {}
    for f in CASES.parent.parent.joinpath("probes-numeric", "cases").glob("t1042-gamma-*.case.json"):
        spec = json.loads(f.read_text())
        out = json.loads(f.with_name(f.name.replace("case.json", "oracle.json")).read_text())["sheets"]
        for sh in spec["sheets"]:
            for cell, formula in sh["cells"].items():
                if str(formula).startswith("=GAMMALN(") and str(formula)[9:-1].replace(".", "").isdigit():
                    v = out.get(sh["name"], {}).get(cell)
                    if v and v.get("t") == "n":
                        gl[float(str(formula)[9:-1])] = v["v"]

    def toms_reduced(a: float, b: float) -> float:
        n, w, a0 = int(a - 1.0), 1.0, a
        for _ in range(n):
            a0 -= 1.0
            w *= a0 / (a0 + b)
        return (B.LN(w) + k.gamln(a0)) + k.algdiv(a0, b)

    return {
        "accurate ln B, rounded": lambda a, b: float(accurate(a, b)),
        "TOMS betaln (Excel LN)": k.betaln,
        "TOMS reduction, w = prod a/(a+b), (lnw + gamln) + algdiv": toms_reduced,
        "gamln(a) + algdiv(a, b), no reduction": lambda a, b: k.gamln(a) + k.algdiv(a, b),
        "(GAMMALN(a) + GAMMALN(b)) - GAMMALN(a+b)": lambda a, b: (gl[a] + gl[b]) - gl[a + b],
        "GAMMALN(a) + (GAMMALN(b) - GAMMALN(a+b))": lambda a, b: gl[a] + (gl[b] - gl[a + b]),
        "(GAMMALN(a) - GAMMALN(a+b)) + GAMMALN(b)": lambda a, b: (gl[a] - gl[a + b]) + gl[b],
    }


def main() -> None:
    rows = grid()
    table = {(a, b): kv for a, b, kv in rows}
    print("k = (Excel's L - fl(accurate L)) / ulp(L); a down, b across")
    print("a\\b  " + " ".join(f"{b:>6}" for b in BS))
    for a in A:
        print(f"{a:<4} " + " ".join(f"{table[a, b]:>6.2f}" for b in BS))
    print("max |k - round(k)|: %.2f" % max(abs(kv - round(kv)) for _, _, kv in rows))
    for name, fn in candidates().items():
        hits = sum(kk(fn(a, b), a, b) == round(kv) for a, b, kv in rows)
        print(f"{hits:>3} of {len(rows)}  {name}")


if __name__ == "__main__":
    main()
