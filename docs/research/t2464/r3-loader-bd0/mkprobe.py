"""T-2513 r3-loader-bd0 probes: BETA.DIST(x,a,b,FALSE) on runs of consecutive doubles placed by the p-side ratio
v = (k - n x)/(k + n x) of Loader's bd0 (k = a-1, n = a+b-2).

  a     (in-sample, series branch): 8 shapes, 4 with n a power of two (n x exact: only the series' own arithmetic
        rounds) and 4 without, at v = -0.30, -0.19, -0.10, +0.10, +0.19, +0.30, 256 doubles each.
  b     (in-sample, integer and half-integer shapes): 10 shapes at v = -0.30, -0.15, +0.15, +0.30, 128 each.
  hold  (hold-out, built before any tuning, scored once): 8 other shapes at v = -0.25, -0.17, -0.07, +0.06, +0.15,
        +0.24, 256 each.
Run starts are irregular (the target x times 1 + 0.000137 i + 0.0000731).  Formula text reuses w2-forensics' exact
encoder.  The meta file holds the block list only (cells are assigned in order).
Usage: mkprobe.py [--write]"""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).parent
sys.path.append(str(HERE.parent / "w2-forensics"))
sys.path.remove(str(HERE)) if str(HERE) in sys.path else None
import make_probes as MP  # noqa: E402  (w2-forensics)

PROBES = {
    "a": (
        [(20.5, 45.5), (40.25, 89.75), (100.5, 157.5), (5.5, 12.5), (20.3, 30.1), (5.3, 9.7), (40.7, 55.3), (100.0, 120.0)],
        (-0.30, -0.19, -0.10, 0.10, 0.19, 0.30),
        256,
    ),
    "b": (
        [(3.0, 3.0), (4.0, 4.0), (5.0, 5.0), (4.0, 6.0), (6.0, 4.0), (3.0, 5.0), (2.0, 5.0), (4.5, 4.5), (10.0, 10.0),
         (6.0, 9.0)],
        (-0.30, -0.15, 0.15, 0.30),
        128,
    ),
    "hold": (
        [(12.7, 33.3), (60.5, 67.5), (24.5, 41.5), (7.5, 9.5), (150.0, 90.0), (3.0, 7.0), (6.0, 6.0), (9.25, 4.75)],
        (-0.25, -0.17, -0.07, 0.06, 0.15, 0.24),
        256,
    ),
}


def x_for(a: float, b: float, v: float) -> float:
    k, n = a - 1, a + b - 2
    return k * (1 - v) / (1 + v) / n


def blocks(name):
    shapes, vs, cnt = PROBES[name]
    out = []
    i = 0
    for a, b in shapes:
        for v in vs:
            x0 = x_for(a, b, v) * (1 + 0.000137 * i + 0.0000731)
            if not 0.02 < x0 < 0.98:
                continue
            out.append([f"{name}({a:g},{b:g})v{v:+.2f}", a, b, x0, cnt])
            i += 1
    return out


def case_id(name):
    return f"t2513-r3-loader-bd0-{name}"


def build(name):
    rows = []
    bl = blocks(name)
    for _, a, b, x0, cnt in bl:
        for x in MP.run(x0, 0, cnt):
            rows.append((x, a, b, False))
    c = MP.case(case_id(name), f"T-2513 r3-loader-bd0 probe {name}: BETA.DIST pdf runs at Loader bd0 ratios "
                f"{PROBES[name][1]} for shapes {PROBES[name][0]}", rows)
    c["tags"] = ["M9", "probe", "statistics", "T-2513", "incomplete-beta"]
    return c, bl


if __name__ == "__main__":
    for name in PROBES:
        c, bl = build(name)
        p = MP.CASES / f"{case_id(name)}.case.json"
        print(case_id(name), len(c["sheets"][0]["cells"]), "exists" if p.exists() else "")
        if "--write" in sys.argv:
            assert not p.exists()
            p.write_text(json.dumps(c, indent=1) + "\n")
            (HERE / f"probe_{name}.meta.json").write_text(json.dumps({"case": case_id(name), "blocks": bl}) + "\n")
