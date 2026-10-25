"""T-term consistency (the s9 decomposition): with P = fl(c LN x) (c = a for mx <= n, -b above), Excel's
ln F + ln x - P = T(x) + noise, T a stored double. T_k = the double nearest tau_k = ln(Excel_k) + ln x_k - P_k
(the gap |tau - T| in units is printed: small means P is right). Then, per candidate form of h L(r), find the C
with st(C - hL_k) == T_k on the most k (interval sweep), i.e. T = fl(C - hL) with one free constant C per run.
Only runs with ulp(T) >= 16 units are informative (below it the +-2 unit noise hides T) and are counted.
Usage: s10_tfit.py SET [min max(d)] [--control=FORM] [--r=mx|q|e|inv]"""
import statistics as S
import sys

import mpmath

import fblib as B

F = B.F
LDB, st = F.LDB, F.st
mpf = mpmath.mpf
U = mpf(2) ** 53
argv = [a for a in sys.argv[1:] if not a.startswith("--")]
SET = argv[0]
MIN = float(argv[1]) if len(argv) > 1 else 0
ctl = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--control=")), None)


def series(r):
    return mpf(float(F.TD.log1p_xl(r)))


def lnfl1p(r):
    return mpf(float(F.LN(st(LDB(1) + LDB(r)))))


def big_side(r):  # -LN(fl(1/fl(1+r)))
    return -mpf(float(F.LN(st(LDB(1) / LDB(st(LDB(1) + LDB(r)))))))


def toms_alnrel(r):
    return mpf(B.kern({}).alnrel(r) if r <= 0.375 else float(F.LN(st(LDB(1) + LDB(r)))))


def l1m(z):
    """-ln(1 - z) by Excel's log1p of -z (series for z <= 0.375, LN(fl(1 - z)) above)."""
    if z <= 0.375:
        return -mpf(float(F.TD.log1p_xl(-z)))
    return -mpf(float(F.LN(st(LDB(1) - LDB(z)))))


def xl(r):
    return series(r) if r <= 0.375 else lnfl1p(r)


FORMS = {
    "h*xl": lambda h, a, b, r: h * xl(r),
    "fl(h*xl)": lambda h, a, b, r: mpf(st(LDB(h) * LDB(float(xl(r))))),
    "fl(a*xl)+fl(b*xl)": lambda h, a, b, r: mpf(st(LDB(a) * LDB(float(xl(r))))) + mpf(st(LDB(b) * LDB(float(xl(r))))),
    "h*LN(fl(1+r))": lambda h, a, b, r: h * lnfl1p(r),
    "h*-LN(1/s)": lambda h, a, b, r: h * (series(r) if r <= 0.375 else big_side(r)),
    "h*alnrel": lambda h, a, b, r: h * toms_alnrel(r),
    "h*exact": lambda h, a, b, r: h * mpmath.log1p(mpf(r)),
    # ln(1 + r) = -ln(1 - z), z = r/(1 + r) the small beta-side variable, by Excel's log1p (series to 0.375)
    "h*-l1p(-z),z=r/s": lambda h, a, b, r: h * l1m(st(LDB(r) / LDB(st(LDB(1) + LDB(r))))),
    "h*-l1p(-z),z=1/(1+1/r)": lambda h, a, b, r: h * l1m(st(LDB(1) / LDB(st(LDB(1) + LDB(st(LDB(1) / LDB(r))))))),
    "h*-l1p(-z),z=r*fl(1/s)": lambda h, a, b, r: h * l1m(st(LDB(r) * LDB(st(LDB(1) / LDB(st(LDB(1) + LDB(r))))))),
}


RFORM = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--r=")), "mx")


def ratio(m, n, x, mx, side):
    """r by form: 'mx' fl(mx/n) | fl(n/mx); 'q' fl(fl(m/n) x) | fl(fl(n/m)/x); 'e' one rounding of the exact
    ratio (mx in extended); 'inv' side 2 as fl(1/fl(mx/n))."""
    M, N, X = LDB(m), LDB(n), LDB(x)
    if RFORM == "mx":
        return st(LDB(mx) / N) if side == 1 else st(N / LDB(mx))
    if RFORM == "q":
        return st(LDB(st(M / N)) * X) if side == 1 else st(LDB(st(N / M)) / X)
    if RFORM == "e":
        return st((M * X) / N) if side == 1 else st(N / (M * X))
    if RFORM == "inv":
        return st(LDB(mx) / N) if side == 1 else st(LDB(1) / LDB(st(LDB(mx) / N)))
    raise ValueError(RFORM)


def pieces(r0, x):
    m, n = r0["d1"], r0["d2"]
    a, b = m / 2, n / 2
    mx = st(LDB(m) * LDB(x))
    side = 1 if mx <= n else 2
    r = ratio(m, n, x, mx, side)
    c = a if side == 1 else -b
    P = mpf(st(LDB(c) * F.LN(x)))
    return a, b, r, P


def best_c(T, hl, ulp):
    """Most k with st(C - hl_k) == T_k for one C: C in [T_k + hl_k - ulp_k/2, T_k + hl_k + ulp_k/2]."""
    ev = []
    for t, v, u in zip(T, hl, ulp):
        ev.append((t + v - u / 2, 0))
        ev.append((t + v + u / 2, 1))
    ev.sort()
    cur = best = 0
    for _, kind in ev:
        cur = cur + 1 if kind == 0 else cur - 1
        best = max(best, cur)
    return best


rows = [r for r in B.load(SET) if max(r["d1"], r["d2"]) >= MIN]
runs: dict = {}
for r in rows:
    runs.setdefault(r["blk"], []).append(r)
tot = {k: 0 for k in FORMS}
N = 0
print(f"# r form {RFORM}")
print(f"# {SET}: per run, gap median |tau - T| (units), ulp(T) (units), then consistent k per form" +
      (f"; CONTROL {ctl}" if ctl else ""))
print("run".ljust(22), "gap ulpT", " ".join(f"{k:>18s}" for k in FORMS))
for blk, rr in runs.items():
    r0 = rr[0]
    h = (r0["d1"] + r0["d2"]) / 2
    taus, Ts, ulps, hls = [], [], [], {k: [] for k in FORMS}
    for r in rr:
        a, b, rat, P = pieces(r0, r["x"])
        if ctl:  # synthetic Excel: T = fl(C0 - hL), C0 = exact constant, value = exp(T + P - ln x)
            X, M, Nn = mpf(r["x"]), mpf(r0["d1"]), mpf(r0["d2"])
            side1 = M * X <= Nn
            rex = M * X / Nn if side1 else Nn / (M * X)
            cex = (a if side1 else -b) * mpmath.log(X)
            K = mpmath.log(r["exact"]) + mpmath.log(X) - cex + h * mpmath.log1p(rex)  # x-free exact constant
            tau = mpf(st(F.R3._ld_of(K) - F.R3._ld_of(FORMS[ctl](h, a, b, rat))))
        else:
            tau = mpmath.log(r["excel"]) + mpmath.log(r["x"]) - P
        T = mpf(float(tau))
        taus.append(tau)
        Ts.append(T)
        ulps.append(mpf(math_ulp) if (math_ulp := __import__("math").ulp(float(T))) else mpf(0))
        for k, fn in FORMS.items():
            hls[k].append(fn(h, a, b, rat))
    if float(ulps[0] * U) < 16:
        continue
    gap = S.median(float(abs(t - T) * U) for t, T in zip(taus, Ts))
    uT = float(ulps[0] * U)
    out = []
    N += len(rr)
    for k in FORMS:
        c = best_c(Ts, hls[k], ulps)
        tot[k] += c
        out.append(f"{c:>18d}")
    print(f"{blk:22s} {gap:4.1f} {uT:5.0f}", " ".join(out))
print("total of", N, " ".join(f"{k}={v}" for k, v in tot.items()))
