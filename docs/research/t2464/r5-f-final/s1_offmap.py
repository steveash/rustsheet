"""T-2586 r5-f-final diagnostics (after the one-shot scores): M-final's within-15-digit rate by |P| = |c ln x| and
|H| = h L(r) bins, on the final hold-out and all earlier sets. Shows where the visible misses live.
Usage: s1_offmap.py"""

from __future__ import annotations

import math

import final_score as FS

M, EL = FS.M, FS.EL
BINS = (0.5, 4, 32, 256, 2048, math.inf)


def pbin(v: float) -> str:
    for i, b in enumerate(BINS):
        if v < b:
            return f"<{b:g}" if i == 0 else f"[{BINS[i - 1]:g},{b:g})"
    return "inf"


def main():
    sets = FS.set_rows()
    sets["final-hold"] = FS.hold_rows()[0]
    for label, pick in (("final-hold", lambda k: k == "final-hold"), ("earlier", lambda k: k != "final-hold")):
        tab: dict = {}
        for nm, rows in sets.items():
            if not pick(nm):
                continue
            for r in rows:
                m, n, x = math.trunc(r["d1"]), math.trunc(r["d2"]), r["x"]
                if isinstance(r["excel"], str) or r["excel"] == 0 or M.rule(x, m, n) is not None:
                    continue
                side, rr = M.SL.side_r(x, float(m), float(n))
                c = m / 2 if side == 1 else n / 2
                P, H = abs(c * math.log(x)), (m + n) / 2 * math.log1p(rr)
                ok = EL.grade(M.value(x, m, n), r["excel"])
                for k in (f"P {pbin(P)}", f"H {pbin(H)}", f"P {pbin(P)} & H {pbin(H)}"):
                    t = tab.setdefault(k, [0, 0, 0, 0])
                    t[0] += 1
                    t[1] += ok[0]
                    t[2] += ok[1]
                    t[3] += ok[2]
        print(f"== {label}: bit-exact / within 4 ulp / within 15 digits of n (rate within 15)")
        for k in sorted(tab, key=lambda s: (s.count("&"), s)):
            n, a, b, c = tab[k]
            if n >= 20:
                print(f"  {k:32s} {a:6d}/{b:6d}/{c:6d} of {n:6d}  ({c / n:.0%})")


if __name__ == "__main__":
    main()
