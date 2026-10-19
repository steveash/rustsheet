# function+ladder rows over the early coefficients (null-space basis builder); env NP, NQ = number of free p / q coefficients from the top; FSIG, STSIG
import numpy as np, mpmath, math, pickle, sys, os as _os

mpmath.mp.dps = 40
WHICH = [("p", i) for i in range(int(_os.environ.get("NP", "5")))] + [
    ("q", i) for i in range(int(_os.environ.get("NQ", "3")))
]


def rows(p, q, sig=float(_os.environ.get("FSIG", "2e-19")), stsig=float(_os.environ.get("STSIG", "0.3"))):
    """[function rows (rel. change of S per ulp / sig, 200 Chebyshev nodes on [2,3]); ladder rows (late stage value change per ulp / stsig)]"""
    nodes = [mpmath.mpf(2.5) - mpmath.mpf(0.5) * mpmath.cos(mpmath.pi * (i + 0.5) / 200) for i in range(200)]
    A = []
    for x in nodes:
        n = mpmath.mpf(0)
        d = mpmath.mpf(0)
        for c in p:
            n = n * x + mpmath.mpf(c)
        for c in q:
            d = d * x + mpmath.mpf(c)
        A.append(
            [
                float(x ** (len(p) - 1 - i) / n * math.ulp(abs(p[i]))) / sig
                if w == "p"
                else float(-(x ** (len(q) - 1 - i)) / d * math.ulp(abs(q[i]))) / sig
                for w, i in WHICH
            ]
        )
    for x0 in (2.0, 2.01, 2.03, 2.06, 2.1, 2.13):
        for w, c, js in (("p", p, range(5, 9)), ("q", q, range(4, 8))):
            sv = 0.0
            for j in range(len(c)):
                sv = sv * x0 + c[j] if j else c[0]
                if j in js:
                    u = math.ulp(abs(sv))
                    A.append(
                        [
                            (math.ulp(abs(c[i])) * x0 ** (j - i) / u / stsig) if (ww == w and i <= j) else 0.0
                            for ww, i in WHICH
                        ]
                    )
    return np.array(A)
