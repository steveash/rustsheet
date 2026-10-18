"""T-1778: the de-trended fine-factor staircase along one long run of consecutive mantissas.

Reviewer tool (research plan, "Reviewer suggestions"). It needs no phase chaining and no alias search
beyond one integer: with hi = fl(H(z)) at a given alias shift and the sawtooth's linear rise removed,
lo - L0 must be flat inside every block and drop by exactly one tooth (K * 2^-53) at each step of fl(V).
Any other jump size, an upward jump, or a second spacing is new information (a term-count change, a
second rounded quantity, an interval boundary). Blocks that contain a jump print as inconsistent.

Usage (repository root, sidecars checked out, mpmath installed):
    python3 docs/research/t1778_staircase.py CASE FIRST_ROW LAST_ROW BLOCK ALIAS_SHIFT
e.g. python3 docs/research/t1778_staircase.py kmyst-erf-wknee1 1 32768 512 0
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

import mpmath as m

m.mp.dps = 30
CASES = Path(__file__).resolve().parents[2] / "eval/suites/probes-numeric/cases"
A = 1.1170334248975802
U = math.ulp(A)
G = m.mpf(2) ** -53
K = m.mpf("1.1366973297689353")
TOOTH = K * G
H = [0, m.mpf("0.14068131264230"), m.mpf("0.0448191707"), m.mpf("0.00180525"), m.mpf("0.00018")]
C = [m.mpf("0.4546572672773"), m.mpf("-0.0598952645"), m.mpf("0.02203")]
D0 = m.mpf("1.07e-16")


def round64(v: m.mpf) -> m.mpf:
    """Round to a 64-bit significand (x87 register precision)."""
    e = int(m.floor(m.log(abs(v), 2)))
    q = m.mpf(2) ** (e - 63)
    return m.nint(v / q) * q


def main() -> None:
    name, r0, r1, blk, shift = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
    cells = json.loads((CASES / f"{name}.case.json").read_text())["sheets"][0]["cells"]
    oracle = json.loads((CASES / f"{name}.oracle.json").read_text())["sheets"]["Calc"]
    z0 = m.mpf(float(cells[f"A{r0}"]) ** 2)
    intervals: list[tuple[m.mpf, m.mpf]] = []
    for r in range(r0, r1 + 1):
        x = float(cells[f"A{r}"])
        e = oracle[f"B{r}"]["v"]
        zq = m.mpf(x * x)
        h = m.mpf(A) + sum(H[k] * zq**k for k in range(1, 5))
        hi = A + (float(m.nint((h - A) / U)) + shift) * U
        coarse = float(round64(m.mpf(x) * hi))  # x87: product to 64 bits, then stored as a double
        rate = C[0] + 2 * C[1] * zq + 3 * C[2] * zq * zq  # dV/dz
        smooth = m.erf(m.mpf(x)) / x + D0 - h + K * rate * (zq - z0)  # L0 plus the sawtooth's rise
        u = math.ulp(e)
        intervals.append(((m.mpf(e) - u / 2 - coarse) / x - smooth, (m.mpf(e) + u / 2 - coarse) / x - smooth))
    print(f"{name} rows {r0}-{r1} block {blk} alias shift {shift}; values in teeth of K*2^-53")
    prev = None
    for b in range(0, len(intervals), blk):
        lo = max(a for a, _ in intervals[b : b + blk])
        hi = min(c for _, c in intervals[b : b + blk])
        mid, width = (lo + hi) / 2 / TOOTH, (hi - lo) / TOOTH
        step = "" if prev is None else f" step {float(mid - prev):+7.3f}"
        flag = "" if width > 0 else "  <- jump inside block"
        print(f"block {b // blk:4d} mid {float(mid):+8.3f} width {float(width):+6.3f}{step}{flag}")
        if width > 0:
            prev = mid


if __name__ == "__main__":
    main()
