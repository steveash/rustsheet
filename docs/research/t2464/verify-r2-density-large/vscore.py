"""Verifier's own implementation + scorer of r2-density-large's LDe density model (from its RESULTS/claim text).

Rounding: fl64 = 64-bit significand (x87 register), st(v) = fl53(fl64(v)) (stored x87 op). LN / EXP are Excel's,
from the engine harness (excelmath.excel), in batches.  Usage: vscore.py [cut=0.2] [series=ext|exact]
"""
from __future__ import annotations
import hashlib, math, sys, pickle
from collections import defaultdict
from decimal import Decimal
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent)); sys.path.insert(0, str(Path(__file__).parent))
import mpmath
import excelmath as XM
import vdata

mpmath.mp.prec = 200
mpf = mpmath.mpf
f64 = XM.fl64
def st(v): return mpf(XM.fl53(XM.fl64(v)))
LN2PI = float(mpmath.log(2 * mpmath.pi))  # nearest double
assert LN2PI == 1.8378770664093456

opts = dict(a.split("=") for a in sys.argv[1:] if "=" in a)
CUT = float(opts.get("cut", "0.2"))
SERIES = opts.get("series", "ext")
LNSIDE = opts.get("ln", "excel")


def stirlerr(v):
    v = mpf(v)
    return mpmath.loggamma(v + 1) - (v + mpf(1) / 2) * mpmath.log(v) + v - mpmath.log(mpmath.sqrt(2 * mpmath.pi))


ARITH = opts.get("arith", "x87")


def bd0_pre(k, M):
    """returns ('ser', value) or ('log', ratio double, M) """
    if ARITH == "exact":  # exact bd0 at the same (stored) M
        K, Mm = mpf(k), mpf(M)
        return ("ser", K * mpmath.log(K / Mm) + Mm - K)
    d = st(k - M); s = st(k + M)
    if abs(d) < mpf(CUT) * s:
        if SERIES == "exact":
            K, Mm = mpf(k), mpf(M)
            return ("ser", K * mpmath.log(K / Mm) + Mm - K)
        v = f64(d / s); acc = f64(d * v); ej = f64(f64(2 * mpf(k)) * v); v2 = f64(v * v)
        for j in range(1, 2000):
            ej = f64(ej * v2)
            a1 = f64(acc + f64(ej / (2 * j + 1)))
            if a1 == acc:
                break
            acc = a1
        return ("ser", acc)
    return ("log", float(st(mpf(k) / mpf(M))))


def main(only=None):
    rows = vdata.load()
    seen, U = set(), []
    for r in rows:
        key = (r["x"], r["a"], r["b"])
        if key in seen: continue
        seen.add(key)
        if only is None or only(r): U.append(r)
    lnreq = set()
    shapes = {}
    for r in U:
        a, b = r["a"], r["b"]
        if (a, b) not in shapes:
            k = float(st(mpf(a) - 1)); s = float(st(mpf(a) + b)); n = float(st(mpf(s) - 2)); nk = float(st(mpf(n) - k))
            s1 = float(st(mpf(s) - 1))
            shapes[(a, b)] = dict(k=k, n=n, nk=nk, s1=s1)
            lnreq |= {k, n, nk, s1}
        sh = shapes[(a, b)]
        q = 1.0 - r["x"]  # double subtraction (exact rounding)
        Mp = float(st(mpf(sh["n"]) * r["x"])); Mq = float(st(mpf(sh["n"]) * q))
        r["_p"] = bd0_pre(sh["k"], Mp); r["_q"] = bd0_pre(sh["nk"], Mq)
        r["_Mp"], r["_Mq"] = Mp, Mq
        for t in (r["_p"], r["_q"]):
            if t[0] == "log": lnreq.add(t[1])
    lnreq = sorted(lnreq)
    LN = dict(zip(lnreq, XM.excel("LN", lnreq)))
    for (a, b), sh in shapes.items():
        k, n, nk = sh["k"], sh["n"], sh["nk"]
        sh["Cs"] = stirlerr(n) - stirlerr(k) - stirlerr(nk)
        sh["lf"] = st(st(st(mpf(LN2PI) + LN[k]) + LN[nk]) - LN[n])
        sh["L5"] = mpf(LN[sh["s1"]])
    def fin(t, kk, M):
        if t[0] == "ser": return t[1]
        tt = st(mpf(kk) * LN[t[1]])
        return f64(tt + f64(mpf(M) - kk))
    Es = []
    for r in U:
        sh = shapes[(r["a"], r["b"])]
        bp = fin(r["_p"], sh["k"], r["_Mp"]); bq = fin(r["_q"], sh["nk"], r["_Mq"])
        X = f64(-bp - bq)
        lc = st(sh["Cs"] + X)
        E = st(sh["L5"] + st(lc - sh["lf"] / 2))
        r["E"] = float(E); Es.append(float(E))
        r["_X"], r["_sh"] = X, sh
    EX = XM.excel("EXP", Es)
    for r, m in zip(U, EX):
        r["model"] = m
        X, A, B = mpf(r["x"]), mpf(r["a"]), mpf(r["b"])
        r["cr"] = float(X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B))
    return U


def w15(m, e):
    if m == e: return True
    E = Decimal(e); M = Decimal(m)
    exp10 = E.adjusted()
    return abs(E - M) <= Decimal(5) * Decimal(10) ** (exp10 - 15)


def tally(rs, key):
    n = len(rs); ex = sum(r[key] == r["excel"] for r in rs)
    w4 = sum(abs(r[key] - r["excel"]) <= 4 * math.ulp(r["excel"]) for r in rs)
    w = sum(w15(r[key], r["excel"]) for r in rs)
    return n, ex, w4, w


def vhold(r):  # verifier's own split, independent of the worker's md5 key
    return hashlib.sha1(f"V|{r['x'].hex()}|{r['a']}|{r['b']}".encode()).digest()[3] & 1


if __name__ == "__main__":
    U = main()
    tag = "_".join(f"{k}{v}" for k, v in opts.items()) or "default"
    pickle.dump([{k: v for k, v in r.items() if not k.startswith("_")} for r in U], open(Path(__file__).parent / f"scored_{tag}.pkl", "wb"))
    groups = defaultdict(list)
    for r in U:
        groups["all"].append(r)
        groups["p1 (own probe)" if r["src"] == "p1" else "round-1"].append(r)
        groups[f"vsplit{vhold(r)}"].append(r)
    # p1 excluding the cut blocks (cut was read from them): cut blocks are the first 12*256 rows A1..A3072
    for r in U:
        if r["src"] == "p1":
            i = int(r["cell"].split("!A")[1])
            groups["p1 minus cut" if i > 3072 else "p1 cut blocks"].append(r)
    print(f"opts {opts}")
    for g, rs in groups.items():
        print(f"{g:18s} model ex/w4/w15 {tally(rs,'model')}   CR {tally(rs,'cr')}")
