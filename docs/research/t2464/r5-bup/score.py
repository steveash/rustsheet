"""T-3568 r5-bup: score n >= 2 L140 bup+bgrat cells as RN(W + D) with Excel's own pieces (probes t3568-r5-bup-*).

At a point (a0, b0, x0, y0) with n = int(b0), c = b0 - n (c = 1, n - 1 at an integer b0), bratio's L140 is
    w = bup(c, a0, y0, x0, n) = I_x0(a0, b0) - I_x0(a0, c);  then bgrat(a0, c, x0, y0, w) adds D = I_x0(a0, c).
bup = ret * S with ret = brcmp1/c = P fl(x0 y0)/c (P = Excel's BETA.DIST(y0, c, a0, FALSE), probe column C, r3-bgrat)
and S = sum_{i<n} d_i, d_0 = 1, d_i = d_{i-1} (apb + i - 1)/(ap1 + i - 1) y0, apb = fl(c + a0), ap1 = fl(c + 1),
with TOMS's k (terms always summed) and the eps = 1e-15 break after it. D = probe column B. Models differ only in
how ret, S and ret*S are rounded; every one is a fixed composition (no constant is fitted).
usage: score.py tune|hold [model ...]    (no model: all of them)"""
from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r5-bgrat-fdf1"))
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
import fmodel  # noqa: E402
from common import CASES, sl, xlk  # noqa: E402

LD = np.longdouble
EPS = 1e-15
mpmath.mp.dps = 50


def num(c):
    return c["v"] if c and c.get("t") == "n" else None


def load(half: str):
    name = f"t3568-r5-bup-{half}"
    out = json.load(open(CASES / f"{name}.oracle.json"))["sheets"]["Calc"]
    rows = []
    for k, region, a0, b0, x0, y0, n, c, dbr in json.load(open(HERE / f"{name}.meta.json")):
        e, d, p = (num(out.get(f"{col}{k}")) for col in "ABC")
        if None in (e, d, p):
            continue
        rows.append(dict(k=k, region=region, a0=a0, b0=b0, x0=x0, y0=y0, n=n, c=c, dbr=dbr.replace("/sw", ""),
                         excel=e, D=d, P=p))
    return rows


def k_terms(c: float, a0: float, x: float, y: float, n: int) -> int:
    """TOMS bup's k: the terms summed before the eps break may apply (bup's a = c, b = a0, x = y0, y = x0)."""
    nm1 = n - 1
    if a0 <= 1.0:
        return 0
    if y <= 1e-4:
        return nm1
    r = (a0 - 1.0) * x / y - c
    if r < 1.0:
        return 0
    return int(r) if r < nm1 else nm1


def bup_sum(c: float, a0: float, x: float, y: float, n: int, arith: str, brk: bool = True):
    """S in the arithmetic named: 'd' doubles in Fortran order, 'w4' r2-bratio's, 'ld' x87 extended, 'ex' exact."""
    apb, ap1 = c + a0, c + 1.0
    k = k_terms(c, a0, x, y, n)
    if arith == "ex":
        d, w = mpmath.mpf(1), mpmath.mpf(1)
    elif arith == "ld":
        d, w = LD(1), LD(1)
    else:
        d, w = 1.0, 1.0
    for i in range(n - 1):
        l = float(i)
        if arith == "d":
            d = (apb + l) / (ap1 + l) * x * d
            w = w + d
        elif arith == "dd":  # d * (ratio * x): the product grouped the other way
            d = d * ((apb + l) / (ap1 + l) * x)
            w = w + d
        elif arith == "w4":
            r = LD(float(LD(apb + l) / LD(ap1 + l)))
            d = float(LD(d) * (r * LD(x)))
            w = float(LD(w) + LD(d))
        elif arith == "ld":
            d = LD(apb + l) / LD(ap1 + l) * LD(x) * d
            w = w + d
        elif arith == "lds":  # extended products, d and w stored to double each step
            d = float(LD(apb + l) / LD(ap1 + l) * LD(x) * LD(d))
            w = float(LD(w) + LD(d))
        else:
            d = mpmath.mpf(apb + l) / mpmath.mpf(ap1 + l) * mpmath.mpf(x) * d
            w = w + d
        if brk and i >= k and d <= EPS * w:
            break
    return w


def ret_of(r, form: str):
    p, x, y, c = r["P"], r["y0"], r["x0"], r["c"]
    if form == "x87":
        return LD(p) * LD(x * y) / LD(c)
    if form == "d":
        return LD(p * (x * y) / c)
    return mpmath.mpf(p) * x * y / c


def model(r, ret_form: str, arith: str, prod: str, brk: bool = True, d_src: str = "xl"):
    """RN(W + D) with W = ret * S rounded as `prod` says ('d': fl(fl(ret) * fl(S)); 'x87': one rounding of the
    extended product; 'ex': exact), D Excel's (d_src 'xl') or exact x0^a0 (d_src 'pow', c = 1 only)."""
    s = bup_sum(r["c"], r["a0"], r["y0"], r["x0"], r["n"], arith, brk)
    ret = ret_of(r, ret_form)
    if prod == "ex":
        w = mpmath.mpf(ret if ret_form == "ex" else float(ret)) * (s if arith == "ex" else mpmath.mpf(float(s)))
    elif prod == "x87":
        w = mpmath.mpf(float(LD(ret) * LD(s)))
    else:
        w = mpmath.mpf(float(float(ret) * float(s)))
    d = r["D"] if d_src == "xl" else float(mpmath.mpf(r["x0"]) ** r["a0"])
    return float(w + d)


def exact(r) -> float:
    return float(mpmath.betainc(r["a0"], r["b0"], 0, r["x0"], regularized=True))


MODELS = {
    "x87 ret, S ld, x87 W": lambda r: model(r, "x87", "ld", "x87"),
    "x87 ret, S d, fl W": lambda r: model(r, "x87", "d", "d"),
    "x87 ret, S dd, fl W": lambda r: model(r, "x87", "dd", "d"),
    "x87 ret, S w4, fl W": lambda r: model(r, "x87", "w4", "d"),
    "x87 ret, S w4, x87 W": lambda r: model(r, "x87", "w4", "x87"),
    "x87 ret, S lds, x87 W": lambda r: model(r, "x87", "lds", "x87"),
    "x87 ret, S d, x87 W": lambda r: model(r, "x87", "d", "x87"),
    "fl ret, S d, fl W": lambda r: model(r, "d", "d", "d"),
    "x87 ret, S ld no break, x87 W": lambda r: model(r, "x87", "ld", "x87", brk=False),
    "exact ret*S (Excel P), + D": lambda r: model(r, "ex", "ex", "ex"),
    "x87 ret, S ld, x87 W, D = x0^a0": lambda r: model(r, "x87", "ld", "x87", d_src="pow") if r["c"] == 1.0 else None,
}


def main() -> None:
    half = sys.argv[1]
    names = sys.argv[2:] or list(MODELS)
    rows = load(half)
    xl = xlk.run([(r["a0"], r["b0"], r["x0"], r["y0"]) for r in rows], fmodel.CFG)
    groups = defaultdict(lambda: defaultdict(list))
    for r, o in zip(rows, xl):
        vals = {m: MODELS[m](r) for m in names}
        vals["M0 (r5 fmodel kernel)"] = o[0]
        vals["CR at the kernel's x0"] = exact(r)
        for key in ((r["region"], r["dbr"]), ("ALL", "")):
            for m, v in vals.items():
                if v is not None:
                    groups[key][m].append((v, r["excel"]))
    allm = [*names, "M0 (r5 fmodel kernel)", "CR at the kernel's x0"]
    print(f"## {half}: exact/w4/w15 (n)\n")
    print("| model | " + " | ".join(f"{k[0]} {k[1]}".strip() for k in sorted(groups)) + " |")
    print("|---|" + "---|" * len(groups))
    for m in allm:
        line = f"| {m} |"
        for k in sorted(groups):
            n, e, f, w = sl.tally(groups[k][m])
            line += f" {e}/{f}/{w} ({n}) |" if n else " - |"
        print(line)


if __name__ == "__main__":
    main()
