"""Per-shape M2 scores and a free-lnB scan (lnB moved by -K..+K ulps; the best shift and its count) on a dataset,
to tell lnB misses from exponent-path misses. Usage: vshape.py dataset [K] [alg] [ar]"""

import collections
import math
import sys

import vlib as V
import vscore as S


def shift(v: float, k: int) -> float:
    for _ in range(abs(k)):
        v = math.nextafter(v, math.inf if k > 0 else -math.inf)
    return v


def main(name, K=12, alg="toms", ar="d"):
    for c in S.GL_CASES:
        V.load_measured_gl(c)
    rows = S.load(name)
    by = collections.defaultdict(list)
    for r in rows:
        by[(r["a"], r["b"])].append(r)
    tot = collections.Counter()
    for (a, b), rs in sorted(by.items()):
        L = V.lnb_m2(a, b, True, alg, ar)
        ex = [r["excel"] for r in rs]
        base = V.score(V.m2(rs, lambda *_: L, ar), ex)[1]
        best = max(((V.score(V.m2(rs, lambda *_, k=k: shift(L, k), ar), ex)[1], -abs(k), k)
                    for k in range(-K, K + 1)))
        reg = V.region(a, b)
        tot[reg, "n"] += len(rs)
        tot[reg, "m2"] += base
        tot[reg, "free"] += best[0]
        flag = "" if base == len(rs) else "  <-"
        print(f"{name} {reg:6s} ({a}, {b}) lnB={L!r}: M2 {base}/{len(rs)}, free lnB best {best[0]} at {best[2]:+d}{flag}")
    for reg in ("min1", "gl", "algdiv"):
        if tot[reg, "n"]:
            print(f"{name} {reg}: n {tot[reg, 'n']}, M2 {tot[reg, 'm2']}, free-lnB ceiling {tot[reg, 'free']}")


if __name__ == "__main__":
    a = sys.argv[1:]
    main(a[0], int(a[1]) if len(a) > 1 else 12, a[2] if len(a) > 2 else "toms", a[3] if len(a) > 3 else "d")
