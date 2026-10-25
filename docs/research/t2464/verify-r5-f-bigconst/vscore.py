"""Verifier's scorer (T-2586 verify-r5-f-bigconst). Own exact density (mpmath, 50 digits), own tally; the worker's
frozen candidate is imported (bcmodel.value) only to evaluate it.
Tally: bit-exact / within 4 ulp / within half a unit of the 15th significant digit of Excel's value.
Classes: x1 (x within 1e-12 of 1), sw (x within 1e-12 relative of d2/d1), rnd (everything else).
Usage: vscore.py CASE [CASE ...]   (probes-numeric case ids)"""

from __future__ import annotations

import importlib
import math
import pickle
import struct
import sys
from pathlib import Path

import mpmath as mp

from vparse import rows

HERE = Path(__file__).resolve().parent
WORKER = HERE.parent / "r5-f-bigconst"
mp.mp.dps = 50
_CF = HERE / "exact_cache.pkl"
_CACHE: dict = pickle.loads(_CF.read_bytes()) if _CF.exists() else {}


def exact(x: float, d1: float, d2: float) -> mp.mpf:
    k = (x, d1, d2)
    if k not in _CACHE:
        X, M, N = mp.mpf(x), mp.mpf(d1), mp.mpf(d2)
        a, b = M / 2, N / 2
        lf = a * mp.log(M / N) + (a - 1) * mp.log(X) - (a + b) * mp.log1p(M * X / N) - mp.log(mp.beta(a, b))
        _CACHE[k] = +mp.exp(lf)
    return _CACHE[k]


def save() -> None:
    _CF.write_bytes(pickle.dumps(_CACHE))


def ordkey(v: float) -> int:
    i = struct.unpack("<q", struct.pack("<d", v))[0]
    return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)


def ulpdist(a: float, b: float) -> int:
    return abs(ordkey(a) - ordkey(b))


def w15(m: float, e: float) -> bool:
    if e == 0:
        return m == 0
    q = mp.mpf(10) ** (mp.floor(mp.log10(abs(mp.mpf(e)))) - 14)
    return abs(mp.mpf(m) - mp.mpf(e)) <= q / 2


def cls(r: dict) -> str:
    if abs(r["x"] - 1) < 1e-12:
        return "x1"
    if abs(r["x"] / (r["d2"] / r["d1"]) - 1) < 1e-12:
        return "sw"
    return "rnd"


def tally(pairs) -> tuple[int, int, int, int]:
    pairs = list(pairs)
    return (sum(m == e for m, e in pairs), sum(ulpdist(m, e) <= 4 for m, e in pairs), sum(w15(m, e) for m, e in pairs),
            len(pairs))


def models() -> dict:
    sys.path.insert(0, str(WORKER))
    bc = importlib.import_module("bcmodel")
    return {
        "CR": lambda r: float(exact(r["x"], r["d1"], r["d2"])),
        "M-sd": lambda r: bc.SD.value(r["x"], r["d1"], r["d2"], "div"),
        "M-sdm": lambda r: bc.SD.value(r["x"], r["d1"], r["d2"], "mulr"),
        "M-bc": lambda r: bc.value(r["x"], r["d1"], r["d2"], "div"),
        "M-bcm": lambda r: bc.value(r["x"], r["d1"], r["d2"], "mulr"),
    }


def fmt(t) -> str:
    return f"{t[0]:5d}/{t[1]:5d}/{t[2]:5d} of {t[3]:5d}"


if __name__ == "__main__":
    MOD = models()
    for case in sys.argv[1:]:
        rs = [r for r in rows(case) if r["excel"] > 0]
        bad = sum(not r["echo_ok"] for r in rs)
        print(f"== {case}: {len(rs)} cells, echo misreads {bad}")
        rs = [r for r in rs if r["echo_ok"]]
        for nm, fn in MOD.items():
            vals = [(float(fn(r)), r["excel"], cls(r), min(r["d1"], r["d2"]) >= 16) for r in rs]
            parts = {"all": [(m, e) for m, e, _, _ in vals]}
            for c in ("x1", "sw", "rnd"):
                sel = [(m, e) for m, e, k, _ in vals if k == c]
                if sel:
                    parts[c] = sel
            print(f"  {nm:6s} " + "  ".join(f"{k}: {fmt(tally(p))}" for k, p in parts.items()))
        save()
