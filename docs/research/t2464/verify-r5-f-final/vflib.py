"""Verifier (T-2586, verify-r5-f-final): frozen-model loader (sha256-checked), own grading, CR baseline, regions."""

from __future__ import annotations

import hashlib
import importlib.util
import math
import pickle
import struct
from decimal import Decimal
from fractions import Fraction
from pathlib import Path

import mpmath as mp

HERE = Path(__file__).resolve().parent
T = HERE.parent
MODEL = T / "r5-f-final" / "fdensity_excel.py"
SHA = "e1b368a23286eb90c44d9f06ae4ff8fb4acbf5dac78a91dd795b01c5502d1173"
TINY = 2.0**-1022


def load_model():
    h = hashlib.sha256(MODEL.read_bytes()).hexdigest()
    assert h == SHA, f"model changed: {h}"
    spec = importlib.util.spec_from_file_location("fdensity_excel_frozen", MODEL)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


# ------------------------------------------------------------------ grading (own)
def _ord(v: float) -> int:
    i = struct.unpack("<q", struct.pack("<d", v))[0]
    return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)


def grade(model, excel) -> tuple[bool, bool, bool]:
    """(bit-exact, <= 4 ulp, within 15 significant digits). Errors and exact zeros must match exactly."""
    if isinstance(model, str) or isinstance(excel, str) or model == 0 or excel == 0:
        s = (type(model) is type(excel)) and model == excel
        return s, s, s
    if not math.isfinite(model):
        return False, False, False
    u = abs(_ord(model) - _ord(excel))
    e = Decimal(excel).adjusted()
    w = abs(Fraction(model) - Fraction(excel)) <= Fraction(1, 2) * Fraction(10) ** (e - 14)
    return u == 0, u <= 4, w


# ------------------------------------------------------------------ rules + correctly rounded density (baseline)
def rules(x: float, d1: float, d2: float):
    m, n = math.trunc(d1), math.trunc(d2)
    if m < 1 or n < 1 or m > 1e10 or n > 1e10 or x < 0:
        return "#NUM!"
    if x == 0:
        return "#NUM!" if m == 1 else (1.0 if m == 2 else 0.0)
    if x < TINY:
        return "#DIV/0!"
    if math.isinf(float(m) * x):
        return "#NUM!"
    return None


_CF = HERE / "cr_cache.pkl"
_CR: dict = pickle.loads(_CF.read_bytes()) if _CF.exists() else {}


def cr_density(x: float, m: int, n: int) -> float:
    k = (x, m, n)
    if k not in _CR:
        mp.mp.dps = 50
        a, b = mp.mpf(m) / 2, mp.mpf(n) / 2
        X = mp.mpf(x)
        lf = (a * mp.log(m) + b * mp.log(n) + (a - 1) * mp.log(X) - (mp.loggamma(a) + mp.loggamma(b) - mp.loggamma(a + b))
              - (a + b) * mp.log(n + m * X))
        v = mp.exp(lf)
        _CR[k] = float(v) if v > mp.mpf(2) ** -1080 else 0.0
    return _CR[k]


def save_cache():
    _CF.write_bytes(pickle.dumps(_CR))


def cr_value(x, d1, d2):
    v = rules(x, d1, d2)
    if v is not None:
        return v
    f = cr_density(x, math.trunc(d1), math.trunc(d2))
    return 0.0 if abs(f) < TINY else f


# ------------------------------------------------------------------ regions (own)
def region(r) -> list[str]:
    x, ex = r["x"], r["excel"]
    m, n = float(math.trunc(r["d1"])), float(math.trunc(r["d2"]))
    if isinstance(ex, str) or ex == 0 or rules(x, r["d1"], r["d2"]) is not None:
        return ["rule"]
    mx = m * x
    if mx <= n:
        side, rr, c = 1, mx / n, m / 2
    else:
        side, rr, c = 2, n / mx, -n / 2
    lo = "r<=.375" if rr <= 0.375 else "r>.375"
    mn = min(m, n)
    dfc = "small" if mn < 5 else ("mid" if mn < 16 else "large")
    P = abs(c * math.log(x))
    pb = "|P|<4" if P < 4 else ("|P|4-32" if P < 32 else ("|P|32-256" if P < 256 else "|P|>=256"))
    return ["value", f"side{side}/{lo}", dfc, f"{dfc}/s{side}/{lo}", pb]


KEYS = ["all", "value", "rule", "side1/r<=.375", "side1/r>.375", "side2/r<=.375", "side2/r>.375", "small", "mid",
        "large", "|P|<4", "|P|4-32", "|P|32-256", "|P|>=256"]


def tally(rows, preds):
    out: dict = {}
    for r, p in zip(rows, preds):
        g = grade(p, r["excel"])
        for k in ["all"] + region(r):
            t = out.setdefault(k, [0, 0, 0, 0])
            t[0] += g[0]
            t[1] += g[1]
            t[2] += g[2]
            t[3] += 1
    return out


def fmt(t, keys=KEYS) -> str:
    return " | ".join(f"{k} {t[k][0]}/{t[k][1]}/{t[k][2]} n{t[k][3]}" for k in keys if k in t)
