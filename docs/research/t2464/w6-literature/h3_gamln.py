"""w6-literature H3: what is ln Gamma(v) for v < 1 inside Excel's ln B?

w2/w3 found that Excel's GAMMALN(v) error is NOT inherited for v < 1 (an accurate value fits better). The
literature offers these structures for ln Gamma(v < 1) inside a beta kernel:
  GL      Excel's own GAMMALN(v)                                   (w1 M1)
  CR      correctly rounded ln Gamma(v)                            (w3's proxy)
  TOMSd   TOMS 708 gamln: v <= 0.8 -> gamln1(v) - log(v); else gamln1(v-0.5-0.5); IEEE double, Excel LN
  TOMSx   the same with every op stored x87 (fl53(fl64))
  TOMSe   the same in x87 extended, one final store
  LNG     log(gamma(v)) (R lgammafn for |v| <= 10), Excel LN of Excel GAMMA
  GL1     GAMMALN(v+1) - LN(v)
Everything else is w1's M1 (TOMS betaln with GL(max), GL(a+b), algdiv for max > 8). Scored on min(a,b) < 1 pdf cells
of beta_exact and w1's probe1, per max bin, with even/odd halves.
"""

import collections
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "w1copy"))
sys.path.insert(0, str(Path(__file__).parent))
import mpmath  # noqa: E402

from forms import ingredients  # noqa: E402
from h1_lnb import hibin  # noqa: E402
from lib import D, R53, R64, exp_v6, fmt, ln64, load_pdf, load_probe, mpf, score, xl  # noqa: E402
from toms_betaln import GL, LN, algdiv  # noqa: E402

P = [
    0.577215664901533,
    0.844203922187225,
    -0.168860593646662,
    -0.780427615533591,
    -0.402055799310489,
    -0.0673562214325671,
    -0.00271935708322958,
]
Q = [
    1.0,
    2.88743195473681,
    3.12755088914843,
    1.56875193295039,
    0.361951990101499,
    0.0325038868253937,
    6.67465618796164e-4,
]
Rr = [
    0.422784335098467,
    0.848044614534529,
    0.565221050691933,
    0.156513060486551,
    0.017050248402265,
    4.97958207639485e-4,
]
Ss = [1.0, 1.24313399877507, 0.548042109832463, 0.10155218743983, 0.00713309612391, 1.16165475989616e-4]


def horner(c, x, r):
    acc = mpf(c[-1])
    for k in reversed(c[:-1]):
        acc = r(r(acc * x) + k)
    return acc


def gamln1(a, r):
    a = mpf(a)
    if a < 0.6:
        w = r(horner(P, a, r) / horner(Q, a, r))
        return r(-a * w)
    x = r(r(a - 0.5) - 0.5)
    w = r(horner(Rr, x, r) / horner(Ss, x, r))
    return r(x * w)


def toms_gamln(v, mode):
    r = {"d": lambda t: mpf(R53(t)), "x": lambda t: mpf(D(t)), "e": R64}[mode]
    if v <= 0.8:
        lv = ln64(v)
        lv = lv if mode == "e" else mpf(R53(lv))
        return float(D(r(gamln1(v, r) - lv)))
    return float(D(gamln1(v - 0.5 - 0.5, r)))


LG = {
    "GL": GL,
    "CR": lambda v: float(mpmath.loggamma(mpf(v))),
    "TOMSd": lambda v: toms_gamln(v, "d"),
    "TOMSx": lambda v: toms_gamln(v, "x"),
    "TOMSe": lambda v: toms_gamln(v, "e"),
    "LNG": lambda v: LN(xl("GAMMA", [v])[0]),
    "GL1": lambda v: float(D(mpf(GL(v + 1.0)) - mpf(LN(v)))),
}


def lnB(a, b, lg):
    lo, hi = min(a, b), max(a, b)
    if hi <= 8.0:
        return float(D(mpf(lg(lo)) + D(mpf(GL(hi)) - mpf(GL(a + b)))))
    return lg(lo) + algdiv(lo, hi, LN)


def m1(rows, ing, lg):
    out, cache = [], {}
    for r, g in zip(rows, ing):
        a, b, x = r["a"], r["b"], r["x"]
        if (a, b) not in cache:
            cache[(a, b)] = mpf(lnB(a, b, lg))
        L = cache[(a, b)]
        ly = mpf(D(g["l1p"])) if x <= 0.375 else g["ly_yd"]
        tx = mpf(D(g["am1"] * g["LNd"]))
        ty = mpf(D(g["bm1"] * ly))
        E = mpf(D(tx - L)) if b == 1.0 else R64(tx + R64(ty - L))
        out.append(exp_v6(E))
    return out


def run(rows, label):
    rows = [r for r in rows if min(r["a"], r["b"]) < 1 and r["a"] != 1 and r["b"] != 1]
    ing = ingredients(rows)
    groups = collections.defaultdict(list)
    for i, r in enumerate(rows):
        groups[hibin(r["a"], r["b"])].append(i)
        lo = min(r["a"], r["b"])
        groups[f"lo={lo:g}"].append(i)
    print(f"== {label}: n={len(rows)}")
    los = sorted({min(r["a"], r["b"]) for r in rows})
    for name, f in LG.items():
        diffs = [(lo, (f(lo) - float(mpmath.loggamma(mpf(lo)))) / 2.220446049250313e-16) for lo in los]
        m = m1(rows, ing, f)
        print(
            f"  {name:6s} all {fmt(score(m, rows))}   lg-CR (units of 2^-52 abs): "
            + " ".join(f"{lo:g}:{d:+.2f}" for lo, d in diffs[:8])
        )
        for gk in sorted(groups):
            idx = groups[gk]
            ev = [i for i in idx if i % 2 == 0]
            od = [i for i in idx if i % 2 == 1]
            s = score(m, rows, idx)
            print(
                f"      {gk:14s} {s[1]}/{s[0]} (e{score(m, rows, ev)[1]} o{score(m, rows, od)[1]}) w4 {s[2]} w15 {s[3]}"
            )
        sys.stdout.flush()


if __name__ == "__main__":
    run(load_pdf(), "beta_exact pdf, min<1")
    run(load_probe("probe1"), "w1 probe1 pdf, min<1")
