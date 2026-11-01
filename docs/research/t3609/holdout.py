"""T-3609 R4.1: hold-out split of the 150 real IMSECH/IMCSCH texts the forms were chosen on.

A text is hold-out when sha256("T-3609:" + cell id)'s first byte is below 77 (about 30%); the split and the margin
(the chosen form's hold-out text-match rate within 5 percentage points of its train rate) were fixed before scoring.
Candidates: the chosen form (IMSECH 1/libm cosh, IMCSCH 1/series sinh, in doubles) and the correctly rounded value
(50-digit 1/cosh, 1/sinh). A text matches when the value rounded to 15 significant digits equals the oracle text's.
Input: lines `FUNC<TAB>ARG<TAB>ORACLE TEXT<TAB>ID` (the suites' oracle texts with 0 < |x| <= 0.01, y = 0).
Usage: python3 holdout.py CELLS.tsv
"""

import hashlib
import math
import sys

import mpmath

mpmath.mp.dps = 50


def sinh_series(x: float) -> float:
    sq, total, term = x * x, x, x
    for k in range(1, 17):
        term = term * sq / ((2 * k) * (2 * k + 1))
        nxt = total + term
        if nxt == total:
            break
        total = nxt
    return total


def match(v: float, text: str) -> bool:
    return float(f"{v:.15g}") == float(text)


def main() -> None:
    rows = []
    for line in open(sys.argv[1]):
        fn, arg, want, cid = line.rstrip("\n").split("\t")
        if fn not in ("IMSECH", "IMCSCH") or "i" in arg or "j" in arg:
            continue
        x = float(arg)
        chosen = 1 / math.cosh(x) if fn == "IMSECH" else 1 / sinh_series(x)
        exact = float(1 / mpmath.cosh(mpmath.mpf(x)) if fn == "IMSECH" else 1 / mpmath.sinh(mpmath.mpf(x)))
        hold = hashlib.sha256(f"T-3609:{cid}".encode()).digest()[0] < 77
        rows.append((fn, hold, match(chosen, want), match(exact, want)))
    for fn in ("IMSECH", "IMCSCH"):
        for name, idx in (("chosen", 2), ("correctly rounded", 3)):
            tr = [r for r in rows if r[0] == fn and not r[1]]
            ho = [r for r in rows if r[0] == fn and r[1]]
            print(f"| {fn} {name} | {sum(r[idx] for r in tr)}/{len(tr)} | {sum(r[idx] for r in ho)}/{len(ho)} |")


if __name__ == "__main__":
    main()
