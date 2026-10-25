"""a9: structure enumeration for log-form F densities with a free per-block constant C.

lnF is written as C + a signed sum of x-dependent stored terms (Excel LN of stored doubles, coefficient products
stored), summed in a given order with stored ('x') or extended ('e') partial sums; C enters first or last; E goes
into EXP V6 extended ('e') or stored ('x').  For each cell the admissible C values form an interval, found by
propagating Excel's admissible E interval backwards through the chain; the score of a structure on a block is the
deepest overlap (cells sharing one C).  A right structure is consistent on (nearly) every cell of a run.
Usage: a9_enum.py SRC [--control FORMULA] [formula ...]   SRC: probe:a | tdens"""

from __future__ import annotations

import collections
import importlib.util
import itertools
import math
import sys

import mpmath

import flib as F

LDB, s = F.LDB, F.st
mpf = mpmath.mpf
_spec = importlib.util.spec_from_file_location("lc_r3lib", F.T / "r3-loader-const" / "r3lib.py")
LC = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(LC)  # r3-loader-const's r3lib (e_interval_ext), loaded under its own name


def S(v):
    return LDB(s(v))


def atoms(x, m, n):
    a, b, h = m / 2, n / 2, (m + n) / 2
    X, M, N = LDB(x), LDB(m), LDB(n)
    mx = S(M * X)
    d = S(N + mx)
    y, q, r = S(mx / d), S(N / d), S(mx / N)
    yw = S(1 / S(1 + S(N / mx)))
    qw = S(1 / S(1 + S(mx / N)))
    LN = F.LN

    def l1p(t):  # Excel's log1p (atanh series for -0.375 <= t < 0, LN(fl(1+t)) otherwise)
        return LDB(F.L.log1p_xl(float(t)))

    return {
        "L1": -LN(x),
        "am1L1": S(LDB(a - 1) * LN(x)),
        "aLmx": S(LDB(a) * LN(mx)),
        "hLd": -S(LDB(h) * LN(d)),
        "hLr1": -S(LDB(h) * LN(s(1 + r))),
        "hLr1b": -S(LDB(h) * LN(s(1 + S(M * S(X / N))))),
        "hl1p": -S(LDB(h) * l1p(r)),
        "aLy": S(LDB(a) * LN(y)),
        "bLq": S(LDB(b) * LN(q)),
        "aLyw": S(LDB(a) * LN(yw)),
        "bLqw": S(LDB(b) * LN(qw)),
        "bl1p": S(LDB(b) * (l1p(-float(y)) if y <= 0.375 else LN(s(1 - y)))),
        "aLr": S(LDB(a) * LN(r)),
        "am1Ly": S(LDB(a - 1) * LN(y)),
        "bm1Lq": S(LDB(b - 1) * LN(q)),
        "Ly": LN(y),
        "hLs1": -S(LDB(h) * LN(s(1 + S(N / mx)))),
        "bp1L1": -S(LDB(b + 1) * LN(x)),
        "bLr1": S(LDB(b) * LN(s(N / mx))),
        "aLyw5": S(LDB(a) * LN(yw)),
        "bLq1": S(LDB(b) * LN(s(S(N / mx) * yw))),
        "Lq": LN(q),
    }


FORMULAS = {
    "A": ["am1L1", "hLr1"],
    "A2": ["am1L1", "hLr1b"],
    "Al": ["am1L1", "hl1p"],
    "B": ["am1L1", "hLd"],
    "C": ["aLy", "bLq", "L1"],
    "Cw": ["aLyw", "bLqw", "L1"],
    "Cl": ["aLy", "bl1p", "L1"],
    "D": ["aLmx", "hLd", "L1"],
    "E": ["aLr", "hLr1", "L1"],
    "El": ["aLr", "hl1p", "L1"],
    "J": ["am1Ly", "bm1Lq", "Ly", "Lq", "L1"],
    "S1": ["hLs1", "bp1L1"],
    "S1r": ["hLs1", "bLr1", "L1"],
    "S1y": ["aLyw5", "bLq1", "L1"],
}


def half_gaps(v: float):
    return (v - math.nextafter(v, -math.inf)) / 2, (math.nextafter(v, math.inf) - v) / 2


def pre_store(lo, hi):
    """Real interval of u with fl(u) in [lo, hi] (lo, hi reals): the doubles inside, widened by half gaps."""
    dlo = float(lo)
    if mpf(dlo) < lo:
        dlo = math.nextafter(dlo, math.inf)
    dhi = float(hi)
    if mpf(dhi) > hi:
        dhi = math.nextafter(dhi, -math.inf)
    if dlo > dhi:
        return None
    return mpf(dlo) - half_gaps(dlo)[0], mpf(dhi) + half_gaps(dhi)[1]


_EI: dict = {}


def e_interval(y: float, fin: str):
    key = (y, fin)
    if key not in _EI:
        if fin == "e":
            _EI[key] = LC.e_interval_ext(y)
        else:
            es = F.R3.eset(y)
            _EI[key] = (mpf(es[0]) - half_gaps(es[0])[0], mpf(es[-1]) + half_gaps(es[-1])[1]) if es else None
    return _EI[key]


def c_interval(terms, y, cpos, acc, fin):
    """Admissible C (real interval) for E = chain(C, terms)."""
    iv = e_interval(y, fin)
    if iv is None:
        return None
    lo, hi = iv
    T = [mpf(float(t)) for t in terms]  # every term is a stored double
    if cpos == "last":  # E = fin(Ssum + C), Ssum the x-dependent sum
        acc_v = T[0]
        for t in T[1:]:
            acc_v = mpf(float(acc_v + t)) if acc == "x" else acc_v + t
        return lo - acc_v, hi - acc_v
    # C first: E = (((C + T1) + T2) + ...); partial sums stored if acc == 'x'; the last sum is E itself
    for t in reversed(T):
        lo, hi = lo - t, hi - t
        if acc == "x" and t is not T[0]:
            r = pre_store(lo, hi)
            if r is None:
                return None
            lo, hi = r
    return lo, hi


def score_block(rows, formula, order, cpos, acc, fin, at):
    ivs = []
    C0 = None
    for r, A in zip(rows, at):
        iv = c_interval([A[k] for k in order], r["excel"], cpos, acc, fin)
        if iv is None:
            ivs.append([])
            continue
        if C0 is None:
            C0 = (iv[0] + iv[1]) / 2
        ivs.append([(float((iv[0] - C0) * 2**53), float((iv[1] - C0) * 2**53))])
    nb, _ = F.R3.best_overlap(ivs)
    return nb


def load(src):
    if src == "tdens":
        rows = F.tdens_rows()
    else:
        import mkprobe

        rows = mkprobe.rows(src.split(":", 1)[1])
    g = collections.defaultdict(list)
    for r in rows:
        g[(r["d1"], r["d2"], r["blk"])].append(r)
    return {k: v for k, v in g.items() if len(v) >= 32}


def main():
    src = sys.argv[1]
    args = sys.argv[2:]
    control = None
    if args and args[0] == "--control":  # synthetic Excel values from formula args[1] (C last, extended)
        control, args = args[1], args[2:]
    names = args or list(FORMULAS)
    blocks = load(src)
    ats = {k: [atoms(r["x"], r["d1"], r["d2"]) for r in rs] for k, rs in blocks.items()}
    if control:
        for k, rs in blocks.items():
            for r, A in zip(rs, ats[k]):
                E = F.R3._ld_of(mpmath.log(r["excel"])) * 0 + LDB(0.123456789)
                for t in FORMULAS[control]:
                    E = E + A[t]
                r["excel"] = F.R3.exp_v6(E)
    ntot = sum(len(v) for v in blocks.values())
    for nm in names:
        best = []
        for order in itertools.permutations(FORMULAS[nm]):
            for cpos, acc, fin in itertools.product(("last", "first"), "xe", "ex"):
                if cpos == "last" and order != tuple(FORMULAS[nm]) and acc == "e":
                    continue  # extended sums commute (to 64 bits)
                tot = 0
                per = {}
                for k, rs in blocks.items():
                    c = score_block(rs, nm, order, cpos, acc, fin, ats[k])
                    per[k] = c
                    tot += c
                best.append((tot, order, cpos, acc, fin, per))
        best.sort(key=lambda t: -t[0])
        for tot, order, cpos, acc, fin, per in best[:3]:
            print(f"{nm:3s} {tot:5d}/{ntot} order {'+'.join(order)} C {cpos} sums {acc} E {fin}")
        tot, order, cpos, acc, fin, per = best[0]
        worst = sorted(per.items(), key=lambda kv: kv[1] / len(blocks[kv[0]]))[:6]
        print(
            "      worst blocks: "
            + ", ".join(f"{k[0]:g},{k[1]:g},{k[2][-6:]} {v}/{len(blocks[k])}" for k, v in worst)
        )
        sys.stdout.flush()


if __name__ == "__main__":
    main()
