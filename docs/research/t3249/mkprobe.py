"""T-3249: oracle sweep of BETA.DIST(x,a,b,FALSE) (Loader's saddle-point density) at large shapes, a + b from 130 to
16385, for T-3250's large-exponent residue. Column B holds F.DIST(x',2a,2b,FALSE) at the mapped point
x' = b*x/(a*(1-x)) (2a and 2b are integers for every shape here) for T-3173.

  fit   a in {128.5, 256.5, 512.5, 1025, 2048.5, 4096.5, 8192.5} x b in {1.5, 2.5, 8.5, 40.5, b = a}, less (1025, 1.5)
        (T-2464's log1p probe already holds that shape): 34 shapes. Per shape, runs of 32 consecutive doubles below the
        mode where ln f = -25, -50, -100, -200, -300, -400, -500, -650, and one run centred on the mode: 9 runs.
  hold  (hold-out, never used for a choice): the fit's a values plus 384.5 and 1536.5, x b in {1.5, 2.5, 4.5, 8.5,
        20.5, 40.5, b = a}, less (1025, 1.5): 62 shapes, runs of 16 doubles at ln f = -35, -75, -150, -250, -350,
        -450, -600 and one run 16 doubles above the mode: 8 runs. The task's sketch (all those shapes at 32 doubles x
        9 runs) is 17,856 rows, above a case's 16,384-row limit, so the hold-out trades run length for shapes.

Every run lies below the mode (x < (a-1)/(a+b-2)): above it ln f falls only like (b-1) ln(1-x), which does not reach
these levels for small b within the doubles below 1. Run starts are the level's x times 1 + u, u uniform in
[1e-5, 2e-4) from the case's own seed, so they are irregular. Formula text reuses w2-forensics' exact encoder.
Usage: mkprobe.py [--write]"""

from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

HERE = Path(__file__).parent
sys.path.append(str(HERE.parent / "t2464" / "w2-forensics"))
import make_probes as MP  # noqa: E402  (w2-forensics)

FIT_A = (128.5, 256.5, 512.5, 1025.0, 2048.5, 4096.5, 8192.5)
CACHED = (1025.0, 1.5)  # T-2464's log1p probe column, owned by T-3250

PROBES = {
    "fit": dict(
        a=FIT_A,
        b=(1.5, 2.5, 8.5, 40.5, None),
        levels=(-25, -50, -100, -200, -300, -400, -500, -650),
        mode_k0=-16,
        cnt=32,
        seed=3249,
    ),
    "hold": dict(
        a=(128.5, 256.5, 384.5, 512.5, 1025.0, 1536.5, 2048.5, 4096.5, 8192.5),
        b=(1.5, 2.5, 4.5, 8.5, 20.5, 40.5, None),
        levels=(-35, -75, -150, -250, -350, -450, -600),
        mode_k0=16,
        cnt=16,
        seed=93249,
    ),
}


def ln_pdf(x: float, a: float, b: float) -> float:
    return (
        (a - 1) * math.log(x)
        + (b - 1) * math.log1p(-x)
        - (math.lgamma(a) + math.lgamma(b) - math.lgamma(a + b))
    )


def x_at_level(a: float, b: float, level: float) -> float | None:
    """The x below the mode where ln f = level, or None if the level is above the mode's."""
    mode = (a - 1) / (a + b - 2)
    if ln_pdf(mode, a, b) <= level:
        return None
    lo, hi = 1e-300, mode
    for _ in range(200):
        mid = math.sqrt(lo * hi) if hi / lo > 4 else (lo + hi) / 2
        if ln_pdf(mid, a, b) < level:
            lo = mid
        else:
            hi = mid
    return hi


def shapes(name: str) -> list[tuple[float, float]]:
    p = PROBES[name]
    out = []
    for a in p["a"]:
        for b in p["b"]:
            s = (a, a if b is None else b)
            if s != CACHED:
                out.append(s)
    return out


def blocks(name: str) -> list[list]:
    p = PROBES[name]
    rng = random.Random(p["seed"])
    out = []
    for a, b in shapes(name):
        for level in p["levels"]:
            x = x_at_level(a, b, level)
            if x is None or x < 1e-300:
                continue
            out.append([f"{name}({a:g},{b:g})L{level}", a, b, x * (1 + rng.uniform(1e-5, 2e-4)), 0, p["cnt"]])
        out.append([f"{name}({a:g},{b:g})mode", a, b, (a - 1) / (a + b - 2), p["mode_k0"], p["cnt"]])
    return out


def case_id(name: str) -> str:
    return f"t3249-beta-large-shape-{name}"


def build(name: str) -> tuple[dict, list]:
    bl = blocks(name)
    rows = []
    for _, a, b, x0, k0, cnt in bl:
        rows += [(x, a, b) for x in MP.run(x0, k0, cnt)]
    assert len(rows) <= 16384, len(rows)
    cells = {}
    for i, (x, a, b) in enumerate(rows, 1):
        cells[f"A{i}"] = f"=BETA.DIST({MP.num(x)},{MP.num(a)},{MP.num(b)},FALSE)"
        xf = b * x / (a * (1 - x))
        cells[f"B{i}"] = f"=F.DIST({MP.num(xf)},{MP.num(2 * a)},{MP.num(2 * b)},FALSE)"
    p = PROBES[name]
    desc = (
        f"T-3249 {name}: BETA.DIST(x,a,b,FALSE) (column A) and F.DIST(b*x/(a*(1-x)),2a,2b,FALSE) (column B) on runs "
        f"of {p['cnt']} consecutive doubles below the mode at ln f = {list(p['levels'])} and at the mode, for "
        f"a in {list(p['a'])} x b in {[('a' if b is None else b) for b in p['b']]} less (1025, 1.5)"
    )
    c = {
        "format": "rustsheet-case/v1",
        "description": desc,
        "tags": ["M9", "probe", "statistics", "T-3249", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    return c, bl


if __name__ == "__main__":
    for name in PROBES:
        c, bl = build(name)
        p = MP.CASES / f"{case_id(name)}.case.json"
        print(
            case_id(name),
            len(bl),
            "runs",
            len(c["sheets"][0]["cells"]),
            "cells",
            "exists" if p.exists() else "",
        )
        if "--write" in sys.argv:
            p.write_text(json.dumps(c, indent=1) + "\n")
            (HERE / f"probe_{name}.meta.json").write_text(
                json.dumps({"case": case_id(name), "blocks": bl}) + "\n"
            )
