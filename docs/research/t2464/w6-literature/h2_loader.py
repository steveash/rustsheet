"""w6-literature H2: Excel's beta density for a, b > 2 as Loader's saddle point (R dbeta's a,b > 2 branch).

Structure from the literature (Loader 2000, "Fast and accurate computation of binomial probabilities"; R nmath
dbeta.c / dbinom.c / stirlerr.c / bd0.c at tags R-2-0-0, R-2-9-0, R-3-6-3):
  pdf = (a+b-1) * dbinom_raw(a-1, a+b-2, x, 1-x)
  dbinom_raw(k, n, p, q): lc = stirlerr(n) - stirlerr(k) - stirlerr(n-k) - bd0(k, n p) - bd0(n-k, n q)
    LF1 (R >= 2.7.2):  lf = ln(2 pi) + ln k + log1p(-k/n);           E = ln(a+b-1) + (lc - 0.5 lf)
    LF0 (R <= 2.7.1):  lf = ln(2 pi) + ln k + ln(n-k) - ln n;        same E
    FX  (R 2.0, Loader paper, non-log): pdf = (a+b-1) * (exp(lc) / sqrt(2 pi k (n-k) / n))
  stirlerr: table for 2n integer and n <= 15; lgamma(n+1) - (n+.5) ln n + n - ln sqrt(2pi) otherwise; series
            S0..S4 with cut-offs 15/35/80/500 above 15.
  bd0: Taylor series in v = (x-np)/(x+np) when |x-np| < 0.1 (x+np), else x ln(x/np) + np - x.
Arithmetic: each op rounded with RND (D = stored x87 fl53(fl64), E = extended fl64, S = plain double).
ln = Excel's LN (FLDLN2/FYL2X model, extended for extended inputs), exp = Excel EXP V6.
Scored per (a, b) block against Excel, with even/odd hold-out halves.
"""

import collections
import csv
import itertools
import math
import sys
from pathlib import Path

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE / "w1copy"))
import mpmath  # noqa: E402

from lib import D, R53, R64, exp_v6, ln64, load_pdf, load_probe, mpf, score, xl  # noqa: E402

SF = [
    0.0,
    0.1534264097200273452913848,
    0.0810614667953272582196702,
    0.0548141210519176538961390,
    0.0413406959554092940938221,
    0.03316287351993628748511048,
    0.02767792568499833914878929,
    0.02374616365629749597132920,
    0.02079067210376509311152277,
    0.01848845053267318523077934,
    0.01664469118982119216319487,
    0.01513497322191737887351255,
    0.01387612882307074799874573,
    0.01281046524292022692424986,
    0.01189670994589177009505572,
    0.01110455975820691732662991,
    0.010411265261972096497478567,
    0.009799416126158803298389475,
    0.009255462182712732917728637,
    0.008768700134139385462952823,
    0.008330563433362871256469318,
    0.007934114564314020547248100,
    0.007573675487951840794972024,
    0.007244554301320383179543912,
    0.006942840107209529865664152,
    0.006665247032707682442354394,
    0.006408994188004207068439631,
    0.006171712263039457647532867,
    0.005951370112758847735624416,
    0.005746216513010115682023589,
    0.005554733551962801371038690,
]
S0, S1, S2, S3, S4 = 1 / 12, 1 / 360, 1 / 1260, 1 / 1680, 1 / 1188
S0, S1, S2, S3, S4 = (float(v) for v in (S0, S1, S2, S3, S4))
M_LN_2PI = 1.837877066409345483560659472811
M_LN_SQRT_2PI = 0.918938533204672741780329736406
M_2PI = 6.283185307179586476925286766559


class Arith:
    def __init__(self, mode, logmode="xl", l1p="cr", lgam="GL"):
        self.mode, self.logmode, self.l1p, self.lgam = mode, logmode, l1p, lgam

    def r(self, v):
        if self.mode == "D":
            return mpf(D(v))
        if self.mode == "E":
            return R64(v)
        return mpf(R53(v))  # plain double

    def ln(self, v):
        v = mpf(v)
        if self.logmode == "cr":
            return self.r(mpmath.log(v))
        lv = ln64(v)
        return lv if self.mode == "E" else mpf(R53(lv))

    def log1p(self, v):
        v = mpf(v)
        if self.l1p == "cr":
            return self.r(mpmath.log1p(v))
        return self.ln(self.r(1 + v))

    def stirlerr(self, n):
        n = mpf(n)
        if n <= 15:
            nn = self.r(n + n)
            if nn == int(nn):
                return mpf(SF[int(nn)])
            n1 = float(self.r(n + 1))
            lg = mpf(xl("GAMMALN", [n1])[0]) if self.lgam == "GL" else self.ln(xl("GAMMA", [n1])[0])
            t = self.r(self.r(n + 0.5) * self.ln(n))
            return self.r(self.r(self.r(lg - t) + n) - M_LN_SQRT_2PI)
        r = self.r
        nn = r(n * n)
        if n > 500:
            return r(r(S0 - r(S1 / nn)) / n)
        if n > 80:
            return r(r(S0 - r(r(S1 - r(S2 / nn)) / nn)) / n)
        if n > 35:
            return r(r(S0 - r(r(S1 - r(r(S2 - r(S3 / nn)) / nn)) / nn)) / n)
        return r(r(S0 - r(r(S1 - r(r(S2 - r(r(S3 - r(S4 / nn)) / nn)) / nn)) / nn)) / n)

    def bd0(self, x, np_):
        r = self.r
        x, np_ = mpf(x), mpf(np_)
        d = r(x - np_)
        if abs(d) < r(0.1 * r(x + np_)):
            v = r(d / r(x + np_))
            s = r(d * v)
            ej = r(r(2 * x) * v)
            v = r(v * v)
            for j in range(1, 1000):
                ej = r(ej * v)
                s1 = r(s + r(ej / (2 * j + 1)))
                if s1 == s:
                    return s1
                s = s1
            return s
        return r(r(r(x * self.ln(r(x / np_))) + np_) - x)

    def dbeta(self, x, a, b, lf_form):
        r = self.r
        x, a, b = mpf(x), mpf(a), mpf(b)
        k = r(a - 1)
        n = r(r(a + b) - 2)
        y = mpf(D(1 - x))
        nk = r(n - k)
        lc = r(
            r(r(r(self.stirlerr(n) - self.stirlerr(k)) - self.stirlerr(nk)) - self.bd0(k, r(n * x)))
            - self.bd0(nk, r(n * y))
        )
        f = r(r(a + b) - 1)
        if lf_form == "FX":
            ff = r(r(r(M_2PI * k) * nk) / n)
            e = mpf(exp_v6(lc))
            p = r(e / r(mpmath.sqrt(ff)))
            return float(D(p * f))
        if lf_form == "LF1":
            lf = r(r(M_LN_2PI + self.ln(k)) + self.log1p(r(-k / n)))
        else:
            lf = r(r(r(M_LN_2PI + self.ln(k)) + self.ln(nk)) - self.ln(n))
        E = r(self.ln(f) + r(lc - r(0.5 * lf)))
        return exp_v6(E)


def load_rows():
    rows = []
    for r in csv.DictReader(open(HERE.parent / "w4-fractions" / "data.tsv"), delimiter="\t"):
        if r["pdf"] in ("", "nan"):
            continue
        a, b = float(r["a"]), float(r["b"])
        if a > 2 and b > 2:
            blk = r["blk"] if r["src"] != "old" else "old"
            rows.append(
                dict(
                    blk=f"{r['src']}:{blk}",
                    x=float(r["x"]),
                    a=a,
                    b=b,
                    excel=float(r["pdf"]),
                    exact=float(mpf(r["pdfx"])),
                )
            )
    for r in load_probe("probe1"):
        if r["a"] > 2 and r["b"] > 2:
            rows.append(
                dict(
                    blk=f"probe1:{r['a']},{r['b']}",
                    x=r["x"],
                    a=r["a"],
                    b=r["b"],
                    excel=r["excel"],
                    exact=r["exact"],
                )
            )
    return rows


def main(limit_per_blk=600):
    rows = load_rows()
    by = collections.defaultdict(list)
    for r in rows:
        by[r["blk"]].append(r)
    # subsample big blocks for speed (deterministic: first limit_per_blk rows)
    sel = []
    for k in sorted(by):
        sel.extend(by[k][:limit_per_blk])
    groups = collections.defaultdict(list)
    for i, r in enumerate(sel):
        g = r["blk"] if not r["blk"].startswith("old") else "old (all a,b>2)"
        groups[g].append(i)
    cr = [r["exact"] for r in sel]
    print(f"n={len(sel)}; CR all {score(cr, sel)}")
    for g in sorted(groups):
        print(f"   CR {g:40s} {score(cr, sel, groups[g])}")
    variants = list(
        itertools.product(["D", "E", "S"], ["LF1", "LF0", "FX"], ["xl"], ["cr", "ln1"], ["GL", "lnG"])
    )
    for mode, lff, lm, l1p, lg in variants:
        if lff != "LF1" and l1p != "cr":
            continue
        A = Arith(mode, lm, l1p, lg)
        m = []
        for r in sel:
            try:
                m.append(A.dbeta(r["x"], r["a"], r["b"], lff))
            except Exception:  # noqa: BLE001
                m.append(math.nan)
        print(f"== {mode} {lff} log={lm} log1p={l1p} lgamma={lg}: all {score(m, sel)}")
        for g in sorted(groups):
            idx = groups[g]
            ev = [i for i in idx if i % 2 == 0]
            od = [i for i in idx if i % 2 == 1]
            s = score(m, sel, idx)
            print(
                f"   {g:40s} {s[1]}/{s[0]} (e{score(m, sel, ev)[1]} o{score(m, sel, od)[1]}) w4 {s[2]} w15 {s[3]}"
            )
        sys.stdout.flush()


if __name__ == "__main__":
    main(int(sys.argv[1]) if len(sys.argv) > 1 else 600)
