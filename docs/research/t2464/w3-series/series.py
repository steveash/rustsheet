"""Power-series candidates for the direct branch, vectorized over cells, in three arithmetic modes:
  'dbl'  every op rounded once to double (SSE),
  'x87'  every op fl53(fl64(.)) (x87 register, stored after every op),
  'ext'  every op in 80-bit extended, nothing stored until the end (register-resident loop).
Each returns S such that I = P*S, with the prefactor P named in the docstring."""

from __future__ import annotations

import numpy as np

LD = np.longdouble


class Ar:
    def __init__(self, mode):
        self.mode = mode
        self.T = np.float64 if mode == "dbl" else LD

    def c(self, v):
        return np.asarray(v, dtype=np.float64).astype(self.T)

    def r(self, v):  # result of one op
        if self.mode == "x87":
            return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)
        return v

    def add(self, a, b):
        return self.r(a + b)

    def sub(self, a, b):
        return self.r(a - b)

    def mul(self, a, b):
        return self.r(a * b)

    def div(self, a, b):
        return self.r(a / b)

    def out(self, v):
        return np.asarray(v, dtype=self.T).astype(np.float64)


EPS = 2.0**-52


def bpser_sum(x, a, b, mode="x87", tol_kind="eps_a", eps=1e-15, nmax=10000, variant="toms"):
    """TOMS 708 bpser loop: c *= (0.5 - b/n + 0.5)*x; w = c/(a+n); sum += w until |w| <= tol.
    Returns S = 1 + a*sum (P = x^a/(a B)). variant 'toms' (0.5-b/n+0.5), 'simple' (1-b/n), 'nb' ((n-b)/n)."""
    A = Ar(mode)
    x, a, b = A.c(x), A.c(a), A.c(b)
    one, half = A.c(1.0), A.c(0.5)
    n = A.c(0.0)
    sm = A.c(np.zeros(np.shape(x)))
    c = A.c(np.ones(np.shape(x)))
    if tol_kind == "eps_a":
        tol = A.div(A.c(eps), a)
    else:
        tol = None
    active = np.ones(np.shape(x), dtype=bool)
    for it in range(nmax):
        n = A.add(n, one)
        if variant == "toms":
            f = A.add(A.sub(half, A.div(b, n)), half)
        elif variant == "simple":
            f = A.sub(one, A.div(b, n))
        else:
            f = A.div(A.sub(n, b), n)
        cn = A.mul(A.mul(c, f), x)
        w = A.div(cn, A.add(a, n))
        sn = A.add(sm, w)
        unchanged = sn == sm
        c = np.where(active, cn, c)
        sm = np.where(active, sn, sm)
        if tol_kind == "eps_a":
            done = np.abs(w) <= tol
        elif tol_kind == "rel":
            done = np.abs(w) <= A.mul(A.c(eps), np.abs(sm))
        elif tol_kind == "unchanged":
            done = unchanged
        active &= ~done
        if not active.any():
            break
    return A.out(A.add(A.mul(a, sm), one)), A.out(sm)


def cephes_pseries_sum(x, a, b, mode="x87", nmax=10000):
    """Cephes incbet pseries: s = 1/a + sum_{n>=1} t_n/(a+n), t_n = prod (i-b)/i * x^n.  I = x^a/B * s."""
    A = Ar(mode)
    x, a, b = A.c(x), A.c(a), A.c(b)
    one = A.c(1.0)
    ai = A.div(one, a)
    u = A.mul(A.sub(one, b), x)
    v = A.div(u, A.add(a, one))
    t1 = v
    t = u
    n = A.c(2.0)
    s = A.c(np.zeros(np.shape(x)))
    z = A.mul(A.c(2.0**-53), ai)  # MACHEP = 2^-53 (Cephes IEEE: 1.11e-16)
    active = np.abs(v) > z
    for _ in range(nmax):
        if not active.any():
            break
        uu = A.div(A.mul(A.sub(n, b), x), n)
        tn = A.mul(t, uu)
        vn = A.div(tn, A.add(a, n))
        sn = A.add(s, vn)
        t = np.where(active, tn, t)
        s = np.where(active, sn, s)
        v = np.where(active, vn, v)
        n = A.add(n, one)
        active &= np.abs(v) > z
    s = A.add(s, t1)
    s = A.add(s, ai)
    return A.out(s)


def accum_sum(x, a, b, mode="x87", term="ratio", nmax=20000):
    """Forward accumulation starting at 1: S_0 = 1, S_n = S_{n-1} + T_n with T_n = a/(a+n) * prod_{i<=n}(i-b)/i x^n.
    term 'ratio': T_n = T_{n-1} * ((n-1-b)*x*(a+n-1)) / (n*(a+n)) (T_1 = a*(1-b)*x/(a+1));
    'c'    : c_n = c_{n-1}*((n-b)/n)*x, T_n = a*c_n/(a+n);
    'c2'   : c_n = c_{n-1}*(n-b)*x/n,   T_n = a*c_n/(a+n);
    'c3'   : c_n = c_{n-1}*x*(n-b)/n,   T_n = c_n*a/(a+n).  Stops when S is unchanged."""
    Ar_ = Ar(mode)
    x, a, b = Ar_.c(x), Ar_.c(a), Ar_.c(b)
    one = Ar_.c(1.0)
    S = Ar_.c(np.ones(np.shape(x)))
    c = Ar_.c(np.ones(np.shape(x)))
    T = None
    active = np.ones(np.shape(x), dtype=bool)
    n = Ar_.c(0.0)
    for it in range(nmax):
        n = Ar_.add(n, one)
        if term == "ratio":
            if it == 0:
                Tn = Ar_.div(Ar_.mul(Ar_.mul(a, Ar_.sub(one, b)), x), Ar_.add(a, one))
            else:
                nm1 = Ar_.sub(n, one)
                num = Ar_.mul(Ar_.mul(Ar_.sub(nm1, b), x), Ar_.add(a, nm1))
                Tn = Ar_.mul(T, Ar_.div(num, Ar_.mul(n, Ar_.add(a, n))))
            T = Tn if T is None else np.where(active, Tn, T)
        else:
            if term == "c":
                cn = Ar_.mul(Ar_.mul(c, Ar_.div(Ar_.sub(n, b), n)), x)
            elif term == "c2":
                cn = Ar_.div(Ar_.mul(Ar_.mul(c, Ar_.sub(n, b)), x), n)
            else:
                cn = Ar_.div(Ar_.mul(Ar_.mul(c, x), Ar_.sub(n, b)), n)
            c = np.where(active, cn, c)
            T = Ar_.div(Ar_.mul(a, c), Ar_.add(a, n)) if term != "c3" else Ar_.mul(c, Ar_.div(a, Ar_.add(a, n)))
        Sn = Ar_.add(S, T)
        done = Sn == S
        S = np.where(active, Sn, S)
        active &= ~done
        if not active.any():
            break
    return Ar_.out(S)


def accum2(x, a, b, mode="dbl", rec="R1", tf="T1", stop="unchanged", eps=2.0 ** -53, nmax=20000, start=1.0):
    """Forward accumulation S = start + sum T_n, c_0 = 1.
    rec: R1 c*((n-b)/n)*x | R2 c*(n-b)*x/n | R3 c*(1-b/n)*x | R4 c*((0.5-b/n)+0.5)*x | R5 c*x*(n-b)/n
         | R6 (c*x)*((n-b)/n) | R7 c*(((n-b)*x)/n)
    tf : T1 (a*c)/(a+n) | T2 a*(c/(a+n)) | T3 c*(a/(a+n)) | T4 (c/(a+n))*a
    stop: 'unchanged' (S_n == S_{n-1}), 'rel' (|T| <= eps*S), 'abs' (|T| <= eps)."""
    A_ = Ar(mode)
    x, a, b = A_.c(x), A_.c(a), A_.c(b)
    one, half = A_.c(1.0), A_.c(0.5)
    S = A_.div(one, a) if isinstance(start, str) else A_.c(np.full(np.shape(x), start))
    c = A_.c(np.ones(np.shape(x)))
    active = np.ones(np.shape(x), dtype=bool)
    n = A_.c(0.0)
    nterms = np.zeros(np.shape(x), dtype=int)
    for _ in range(nmax):
        n = A_.add(n, one)
        nb = A_.sub(n, b)
        if rec == "R1": cn = A_.mul(A_.mul(c, A_.div(nb, n)), x)
        elif rec == "R2": cn = A_.div(A_.mul(A_.mul(c, nb), x), n)
        elif rec == "R3": cn = A_.mul(A_.mul(c, A_.sub(one, A_.div(b, n))), x)
        elif rec == "R4": cn = A_.mul(A_.mul(c, A_.add(A_.sub(half, A_.div(b, n)), half)), x)
        elif rec == "R5": cn = A_.div(A_.mul(A_.mul(c, x), nb), n)
        elif rec == "R6": cn = A_.mul(A_.mul(c, x), A_.div(nb, n))
        else: cn = A_.mul(c, A_.div(A_.mul(nb, x), n))
        an = A_.add(a, n)
        if tf == "T0": T = A_.div(cn, an)
        elif tf == "T1": T = A_.div(A_.mul(a, cn), an)
        elif tf == "T2": T = A_.mul(a, A_.div(cn, an))
        elif tf == "T3": T = A_.mul(cn, A_.div(a, an))
        else: T = A_.mul(A_.div(cn, an), a)
        Sn = A_.add(S, T)
        if stop == "unchanged": done = Sn == S
        elif stop == "rel": done = np.abs(T) <= A_.mul(A_.c(eps), np.abs(Sn))
        else: done = np.abs(T) <= A_.c(eps)
        c = np.where(active, cn, c)
        S = np.where(active, Sn, S)
        nterms += active
        active &= ~done
        if not active.any():
            break
    return A_.out(S), nterms
