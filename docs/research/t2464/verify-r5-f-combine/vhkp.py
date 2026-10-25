"""verify-r5-f-combine: independent spot check of "HKP11 fits 0 runs with K_hi free and the remainder free".
HKP11: S1 = x87(K_hi + H), S2 = x87(S1 + P), E = fl64(S2 + R), F = x87(EXP64(E) / x)  (vmodel's pieces).
Per cell, the admissible E interval (own bisection on the final step, 1e-3 units of 2^-53); per K_hi double in
fl(K_sd) +- J ulps, the deepest overlap of the remainder intervals. Positive control: Excel replaced by HKP11 at
K_hi = fl(K_sd) + 3 ulps, R = (K_sd - K_hi) + 4.7 units.
Usage: vhkp.py CASE_ID RUNLEN RUN_INDEX..."""

from __future__ import annotations

import math
import sys

import mpmath

import vmodel as VM
import vparse as VP
import excelmath as XM

mpf, U = mpmath.mpf, mpmath.mpf(2) ** -53
J = 12


def pieces(r):
    x, m, n = r["x"], r["d1"], r["d2"]
    side, rr = VM.side_r(x, m, n)
    a, b, h = m / 2, n / 2, (m + n) / 2
    s = XM.x87("+", 1.0, rr)
    P = (1 if side == 1 else -1) * XM.fl53(mpf(a if side == 1 else b) * mpf(VM.LN(x)))
    H = -XM.fl64(mpf(h) * mpf(VM.LN(s)))
    return side, mpf(P), H


def fin(E, x):
    return XM.fl53(XM.fl64(VM._exp64(XM.fl64(E)) / mpf(x)))


def e_interval(E0, x, y, W=4096.0):
    """[lo, hi] in units around E0 with fin(E0 + u) == y, or None (fin is monotone in E)."""
    def f(u):
        return fin(E0 + mpf(u) * U, x)

    lo, hi = -W, W
    if not (f(lo) <= y <= f(hi)):
        return None
    a, b = lo, hi  # find a point inside
    while b - a > 1e-3:
        c = (a + b) / 2
        if f(c) < y:
            a = c
        elif f(c) > y:
            b = c
        else:
            break
    else:
        return None
    mid = (a + b) / 2
    p, q = lo, mid
    while q - p > 1e-3:
        c = (p + q) / 2
        p, q = (c, q) if f(c) < y else (p, c)
    left = q
    p, q = mid, hi
    while q - p > 1e-3:
        c = (p + q) / 2
        p, q = (p, c) if f(c) > y else (c, q)
    return left, p


def best_overlap(ivs):
    ev = sorted([(lo, 0) for lo, _ in ivs] + [(hi, 1) for _, hi in ivs])
    best = cur = 0
    for _, t in ev:
        cur += 1 if t == 0 else -1
        best = max(best, cur)
    return best


def scan(cells, Ksd):
    """max over K_hi of the cells reproduced by one remainder."""
    k0 = XM.fl53(Ksd)
    best = 0
    for j in range(-J, J + 1):
        khi = k0
        for _ in range(abs(j)):
            khi = math.nextafter(khi, math.inf if j > 0 else -math.inf)
        ivs = []
        for (side, P, H, E0, iv) in cells:
            S2 = XM.fl53(XM.fl64(mpf(XM.fl53(XM.fl64(mpf(khi) + H))) + P))
            base = float((E0 - S2) / U)
            ivs.append((base + iv[0], base + iv[1]))
        best = max(best, best_overlap(ivs))
    return best


def main():
    case, runlen = sys.argv[1], int(sys.argv[2])
    rows = VP.rows(case)
    rows.sort(key=lambda r: int(r["cell"][1:]))
    for ri in map(int, sys.argv[3:]):
        rs = rows[ri * runlen : (ri + 1) * runlen]
        VM.prefetch_ln([v for r in rs for v in VM.ln_inputs(r["x"], r["d1"], r["d2"])])
        m, n = rs[0]["d1"], rs[0]["d2"]
        for label in ("excel", "plant"):
            cells = []
            for r in rs:
                side, P, H = pieces(r)
                Ksd = VM.K_sd(m, n, side)
                y = r["excel"]
                if label == "plant":
                    khi = XM.fl53(Ksd)
                    for _ in range(3):
                        khi = math.nextafter(khi, math.inf)
                    S2 = XM.fl53(XM.fl64(mpf(XM.fl53(XM.fl64(mpf(khi) + H))) + P))
                    y = fin(mpf(S2) + (Ksd - mpf(khi)) + 4.7 * U, r["x"])
                E0 = VM.K_sd(m, n, side) + P + H  # reference point; the interval is relative to it
                iv = e_interval(E0, r["x"], y)
                if iv is not None:
                    cells.append((side, P, H, E0, iv))
            print(f"run {ri} ({m:g},{n:g}) side {cells[0][0] if cells else '?'} {label}: "
                  f"{scan(cells, Ksd)}/{len(rs)} (admissible {len(cells)})")
            sys.stdout.flush()


if __name__ == "__main__":
    main()
