"""Full x87-stored simulation of l = log1p(-p) candidates of the form 'r = a/(a+2); l = 2r + correction(r)',
scored bit-exact against pinned l on p in [1e-4, 0.375], per -r bin, with an even/odd hold-out split by row index.
Usage: gen_scan.py [filter-substring]"""
import math
import sys

import pin
import r2lib as R
from x87f import x87 as X

M, A_, D_ = (lambda a, b: X("*", a, b)), (lambda a, b: X("+", a, b)), (lambda a, b: X("/", a, b))


def horner(cs, z):
    s = cs[-1]
    for c in reversed(cs[:-1]):
        s = A_(M(s, z), c)
    return s


def taylor_cs(N, scale=1.0):
    return [scale / (2 * k + 3) for k in range(N + 1)]


def make(kind, N):
    def f(r):
        t2 = M(r, r)
        r2 = 2 * r
        if kind == "S":  # 2r + 2r*(t2*S), S = 1/3 + t2/5 + ...
            return A_(r2, M(r2, M(t2, horner(taylor_cs(N), t2))))
        if kind == "Sb":  # 2r + (2r*t2)*S
            return A_(r2, M(M(r2, t2), horner(taylor_cs(N), t2)))
        if kind == "Q":  # 2r + r*(t2*Q), Q = 2/3 + 2t2/5 + ...
            return A_(r2, M(r, M(t2, horner(taylor_cs(N, 2.0), t2))))
        if kind == "H":  # 2*(r + r*(t2*S))
            return 2 * A_(r, M(r, M(t2, horner(taylor_cs(N), t2))))
        if kind == "W":  # 2r * W, W = 1 + t2/3 + ... (full Horner incl. 1)
            return M(r2, horner([1.0] + taylor_cs(N), t2))
        if kind == "Wr":  # r * (2 + 2t2/3 + ...)
            return M(r, horner([2.0] + taylor_cs(N, 2.0), t2))
        if kind == "rev":  # sum of terms r^(2k+1)/(2k+1) smallest first, then *2
            terms = []
            p = r
            for k in range(N + 2):
                terms.append(D_(p, float(2 * k + 1)))
                p = M(p, t2)
            s = terms[-1]
            for tm in reversed(terms[:-1]):
                s = A_(s, tm)
            return 2 * s
        if kind == "fwdtail":  # r + (sum_{k>=1} terms forward), then *2
            p = M(r, t2)
            s = D_(p, 3.0)
            for k in range(2, N + 2):
                p = M(p, t2)
                s = A_(s, D_(p, float(2 * k + 1)))
            return 2 * A_(r, s)
        raise KeyError(kind)
    return f


CANDS = {f"{k} N={N}": make(k, N) for k in ("S", "Sb", "Q", "H", "W", "Wr", "rev", "fwdtail") for N in (10, 11, 12, 13, 16)}


def load_rows():
    rows = []
    for nm in ("runs", "grid", "hold"):
        for i, (p, n, k, l) in enumerate(pin.load(nm)):
            if k in ("none", "multi") or not (1e-4 <= p <= 0.375):
                continue
            rows.append((p, R.r_of(p, "x"), l, (i % 2, nm)))
    return rows


EDGES = [0, 0.05, 0.1, 0.15, 0.2, 0.2308]


def score(f, rows):
    per = [[0, 0] for _ in EDGES[:-1]]
    half = [[0, 0], [0, 0]]
    miss = []
    for p, r, l, (h, nm) in rows:
        j = max(i for i in range(len(EDGES) - 1) if -r >= EDGES[i])
        ok = f(r) == l
        per[j][0] += ok
        per[j][1] += 1
        half[h][0] += ok
        half[h][1] += 1
        if not ok:
            miss.append((p, nm))
    return per, half, miss


if __name__ == "__main__":
    flt = sys.argv[1] if len(sys.argv) > 1 else ""
    rows = load_rows()
    print("%-16s" % "cand" + "".join("%12s" % f"<{e}" for e in EDGES[1:]) + "      even        odd")
    for nm, f in CANDS.items():
        if flt not in nm:
            continue
        per, half, _ = score(f, rows)
        print("%-16s" % nm + "".join("%12s" % f"{a}/{b}" for a, b in per) + "".join("%11s" % f"{a}/{b}" for a, b in half))
