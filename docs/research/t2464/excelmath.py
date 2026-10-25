"""T-2464 shared helpers: Excel's own elementary functions, evaluated by the engine, and exact x87 rounding.

The engine's EXP (x87_exp.rs, model V6), LN (x87_log10.rs, FLDLN2/FYL2X), GAMMALN (gammaln.rs, T-1941) and GAMMA
(T-2129) are bit-exact models of Excel's. `excel(name, xs)` evaluates `=NAME(x)` for each x in one recalculation.
Run with bindings/python/.venv/bin/python (make py-bindings builds it).

x87 helpers: `fl64(v)` rounds an mpmath value to a 64-bit significand, `fl53(v)` to a double, and `x87(op, a, b)`
is fl53(fl64(a op b)), the stored x87 operation of the playbook (section 1, item 1).
"""
from __future__ import annotations

import math
from functools import lru_cache

import os
import struct
import subprocess
import tempfile

import mpmath

mpmath.mp.prec = 300


def round_bits(v: mpmath.mpf, bits: int) -> mpmath.mpf:
    if v == 0:
        return mpmath.mpf(0)
    s, man, exp, bc = v._mpf_
    if bc <= bits:
        return v
    sh = bc - bits
    q = man >> sh
    r = man - (q << sh)
    half = 1 << (sh - 1)
    if r > half or (r == half and q & 1):
        q += 1
    return mpmath.mpf((s, q, exp + sh, q.bit_length()))


def fl64(v) -> mpmath.mpf:
    return round_bits(mpmath.mpf(v), 64)


def fl53(v) -> float:
    return float(round_bits(mpmath.mpf(v), 53))


def x87(op: str, a, b) -> float:
    """fl53(fl64(a op b)) for doubles a, b (op in + - * /)."""
    A, B = mpmath.mpf(a), mpmath.mpf(b)
    r = {"+": A + B, "-": A - B, "*": A * B, "/": A / B}[op]
    return fl53(fl64(r))


def ulps(got: float, want: float) -> float:
    """Signed distance in ulps of `want`."""
    if got == want:
        return 0.0
    return (got - want) / math.ulp(want) if want != 0 else math.inf


HARNESS = os.environ.get("T2464_HARNESS", os.path.join(os.path.dirname(__file__), ".harness-bin"))


def _hex(x: float) -> str:
    return f"{struct.unpack('<Q', struct.pack('<d', float(x)))[0]:016x}"


def excel(name: str, xs: list[float]) -> list[float]:
    """Excel's NAME(x) for each double x: NAME in EXP, LN, LOG10, GAMMALN, GAMMA (the engine's bit-exact models,
    run by the local test harness, one process per call)."""
    with tempfile.TemporaryDirectory() as d:
        inp, out = os.path.join(d, "in"), os.path.join(d, "out")
        with open(inp, "w") as f:
            for x in xs:
                f.write(f"{name} {_hex(x)}\n")
        env = dict(os.environ, T2464_IN=inp, T2464_OUT=out)
        subprocess.run([HARNESS, "--ignored", "--exact", "functions::statistical::t2464_harness::t2464_kernels",
                        "--quiet"], env=env, check=True, capture_output=True)
        with open(out) as f:
            return [struct.unpack("<d", struct.pack("<Q", int(line, 16)))[0] for line in f.read().split()]


@lru_cache(maxsize=None)
def excel1(name: str, x: float) -> float:
    return excel(name, [x])[0]
