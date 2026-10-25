"""Print probe rows of a section (in-sample 'a' only): x, df, Excel's B (C/2^600, D twin), the exact density.
Usage: s3_look.py KIND [KIND ...]"""
import sys

import mpmath

import edgeprobe as MK

mpmath.mp.prec = 120


def v(c):
    return None if c is None else (c["v"] if c.get("t") == "n" else c["v"])


def exact(o):
    import math
    x, m, n = o["x"], math.trunc(o["d1"]), math.trunc(o["d2"])
    if not x or x <= 0 or m < 1 or n < 1:
        return None
    try:
        return mpmath.exp(mpmath.mpf(MK._lnf(x, m, n)))
    except Exception:  # noqa: BLE001
        return None


for o in MK.rows("a"):
    if o["kind"] not in sys.argv[1:]:
        continue
    echo = v(o["A"])
    bad = "" if (o["x"] is None or echo == o["x"]) else f" ECHO {echo!r}"
    C = v(o["C"])
    Cs = f" C/2^600={C / 2**600 if isinstance(C, float) else C!r}" if o["scale"] else ""
    D = f" D={v(o['D'])!r}" if o["twin"] else ""
    print(f"{o['row']:5d} {o['kind']:6s} x={o['x']!r:24} {o['d1s']},{o['d2s']} B={v(o['B'])!r}{Cs}{D} ex={mpmath.nstr(exact(o), 17) if exact(o) else '-'}{bad}")
