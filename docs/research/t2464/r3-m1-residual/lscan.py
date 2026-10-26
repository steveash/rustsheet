"""Per-shape lnB scan: for each shape (a, b), score M1's exponent path with L = lnb_base moved by j ulps, j in
[-W, W], and with the mode given. If one j reaches ~100% the exponent path is right and only lnB is off; a flat
plateau below 100% means the miss is in tx, ty or the sum.  Usage: lscan.py old|probe1|log1p|<own probe> [mode] [W]
"""

import collections
import sys

import m1lib as M


def rows_of(name):
    return {"old": M.load_old, "probe1": M.load_probe1, "log1p": M.load_log1p_rows}.get(
        name, lambda: M.load_own(name)
    )()


def scan(rows, mode="m1", W=12, lnb=M.lnb_base):
    by = collections.defaultdict(list)
    for r in rows:
        by[(r["a"], r["b"])].append(r)
    out = {}
    for sh, rs in sorted(by.items()):
        L0 = lnb(*sh)
        pc = []
        for r in rs:
            if "l" in r:  # r2-log1p rows: a = 1, ty = n*l exact
                pc.append((0.0, M.sx("*", r["b"] - 1.0, r["l"])))
            else:
                pc.append(M.pieces(r["x"], r["a"], r["b"]))
        hits = {}
        for j in range(-W, W + 1):
            L = M.nb(L0, j)
            hits[j] = sum(
                M.combine(tx, ty, L, sh[0], sh[1], mode) == r["excel"] for (tx, ty), r in zip(pc, rs)
            )
        out[sh] = (len(rs), hits, L0)
    return out


if __name__ == "__main__":
    name = sys.argv[1]
    mode = sys.argv[2] if len(sys.argv) > 2 else "m1"
    W = int(sys.argv[3]) if len(sys.argv) > 3 else 12
    res = scan(rows_of(name), mode, W)
    tot0 = totb = n = 0
    for sh, (cnt, hits, L0) in res.items():
        jb = max(hits, key=lambda j: (hits[j], -abs(j)))
        tot0 += hits[0]
        totb += hits[jb]
        n += cnt
        top = sorted(hits.items(), key=lambda kv: -kv[1])[:4]
        print(
            f"a={sh[0]!r:22} b={sh[1]!r:22} n={cnt:5d} L0={L0!r:24} at0={hits[0]:5d} best j={jb:+d} {hits[jb]:5d}  "
            f"top {top}"
        )
    print(f"total n={n} at j=0 {tot0}, per-shape best {totb}")
