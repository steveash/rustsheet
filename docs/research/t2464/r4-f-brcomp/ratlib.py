"""T-2586 r4-f-brcomp: the log-form ("ratio") family of F density chains, with the term forms r3's c1 did not try.
With mx = fl(m x), h = a + b, r = the smaller of fl(mx/n) (side 1) and fl(n/mx) (side 2):
  ln F = K + P(x) - h L(r)     (P = the x power: side 1 (a-1) ln x, side 2 -(b+1) ln x, up to constants)
Knobs (dict o):
  L   'xl' Excel log1p (series to 0.375, LN(fl(1+r)) above) | 'ln' LN(fl(1+r)) always
  hl  'st' fl(h*L) | 'ext' h*L in extended | 'split' fl(a*L) + fl(b*L) | 'ab' a*L, b*L each stored, summed extended
  pw  power-term form, side 1 / side 2:
      'x'   fl((a-1) LN x)              / fl(-(b+1) LN x)
      'r'   fl(a LN r) - LN x            / fl(b LN r) - LN x
      'rdiv' fl(a LN r), value / x       / fl(b LN r), value / x
      'mx'  fl(a LN mx) - LN x           / -fl(b LN mx) - LN x
      'mx1' fl((a-1) LN mx)              / -fl((b+1) LN mx)
      'exact' the exact power term (isolates L's contribution)
      'xd'  fl(a LN x), value / x        / fl(-b LN x), value / x
      'xn'  fl(a LN x) - LN x            / fl(-b LN x) - LN x
  sum 'ext' (x87 register) | 'st' (each partial sum stored), order K + P - hL
  grp 'none' | 'kl' fl(K - hL) + P (s7_steps: Excel's ln F moves in quanta of two stored terms, one the size of
      K - hL) | 'kl2' fl(fl(K) - hL) + P
  est 'ext' E unstored into EXP | 'st' E stored
  kside (pw xd/xn only) 'exact' | 'st' K = -lnB + fl(c' LN(fl(m/n))), c' = a (side 1), -b (side 2) | 'ln' the
      same product unstored (s12: Excel's offset differs by side when m != n, by about ulp(b ln(m/n)))
K is exact (round 3 fact 1), computed as ln F(x) minus the exact x-dependent part."""

from __future__ import annotations

import functools

import mpmath

import fblib as B

F = B.F
LDB, st = F.LDB, F.st
mpf = mpmath.mpf


def L_of(r: float, mode: str):
    if mode == "xl" and r <= 0.375:
        return LDB(F.TD.log1p_xl(r))
    return F.LN(st(LDB(1) + LDB(r)))


def LNd(v: float):
    return F.LN(v)


@functools.lru_cache(maxsize=None)
def _lnB(m, n):
    return mpmath.log(mpmath.beta(mpf(m) / 2, mpf(n) / 2))


def value(x: float, m: float, n: float, o: dict) -> float:
    a, b, h = m / 2, n / 2, (m + n) / 2
    X, M, N = mpf(x), mpf(m), mpf(n)
    mx = st(LDB(m) * LDB(x))
    if mx <= n:
        side, r = 1, st(LDB(mx) / LDB(n))
    else:
        side, r = 2, st(LDB(n) / LDB(mx))
    L = L_of(r, o.get("L", "xl"))
    hlm = o.get("hl", "ext")
    if hlm == "st":
        hl = LDB(st(LDB(h) * L))
    elif hlm == "split":
        hl = LDB(st(LDB(st(LDB(a) * L)) + LDB(st(LDB(b) * L))))
    elif hlm == "ab":
        hl = LDB(st(LDB(a) * L)) + LDB(st(LDB(b) * L))
    else:
        hl = LDB(h) * L
    pw = o.get("pw", "x")
    lnfx = (M / 2) * mpmath.log(M * X / (M * X + N)) + (N / 2) * mpmath.log(N / (M * X + N)) - mpmath.log(X)
    lnf = lnfx - _lnB(m, n)
    # exact x-dependent pieces (to form the exact constant K for each form)
    Lr_ex = h * mpmath.log1p(mpf(m) * X / N if side == 1 else N / (M * X))
    div = False
    if pw == "exact":
        P_ex = ((a - 1) if side == 1 else -(b + 1)) * mpmath.log(X)
        P = F.R3._ld_of(P_ex)
    elif pw == "x":
        c = (a - 1) if side == 1 else -(b + 1)
        P_ex = c * mpmath.log(X)
        P = LDB(st(LDB(c) * LNd(x)))
    elif pw in ("xd", "xn"):  # fl(a LN x) / fl(-b LN x), then the 1/x as a value division (xd) or -LN x (xn)
        c = a if side == 1 else -b
        P_ex = c * mpmath.log(X)
        P = LDB(st(LDB(c) * LNd(x)))
        if pw == "xn":
            P_ex -= mpmath.log(X)
            P = P - LNd(x)
        else:
            div = True
    elif pw in ("r", "rdiv"):
        c = a if side == 1 else b
        rex = M * X / N if side == 1 else N / (M * X)
        P_ex = c * mpmath.log(rex)
        P = LDB(st(LDB(c) * LNd(r)))
        if pw == "r":
            P_ex -= mpmath.log(X)
            P = P - LNd(x)
        else:
            div = True
    elif pw == "mx":
        c = a if side == 1 else -b
        P_ex = c * mpmath.log(M * X) - mpmath.log(X)
        P = LDB(st(LDB(c) * LNd(mx))) - LNd(x)
    elif pw == "mx1":
        c = (a - 1) if side == 1 else -(b + 1)
        P_ex = c * mpmath.log(M * X)
        P = LDB(st(LDB(c) * LNd(mx)))
    else:
        raise ValueError(pw)
    K_ex = lnf - P_ex + Lr_ex + (mpmath.log(X) if div else 0)
    K = F.R3._ld_of(K_ex)
    ks = o.get("kside", "exact")
    if ks != "exact" and pw in ("xd", "xn"):  # K = -lnB + c' ln(m/n), c' = a | -b, with the ln(m/n) term stored
        cp = a if side == 1 else -b
        mn = st(LDB(m) / LDB(n))
        if ks == "st":  # fl(c' LN(fl(m/n)))
            t = LDB(st(LDB(cp) * F.LN(mn)))
        elif ks == "ln":  # c' LN(fl(m/n)) in extended
            t = LDB(cp) * F.LN(mn)
        else:
            raise ValueError(ks)
        K = F.R3._ld_of(-_lnB(m, n)) + t
    grp = o.get("grp", "none")
    if grp == "kl":  # the constant and the h L term are one stored double: fl(K - h L) + P
        E = LDB(st(K - hl)) + P
    elif grp == "kl2":  # fl(fl(K) - hl) + P
        E = LDB(st(LDB(st(K)) - hl)) + P
    elif o.get("sum", "ext") == "st":
        E = LDB(st(LDB(st(K + P)) - hl))
    else:
        E = K + P - hl
    if o.get("est", "ext") == "st":
        E = LDB(st(E))
    v = F.R3.exp_v6(E)
    return st(LDB(v) / LDB(x)) if div else v
