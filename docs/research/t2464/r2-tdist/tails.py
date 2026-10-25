"""Compose T.DIST.RT / T.DIST.2T / TDIST / T.DIST(TRUE) from r2-bratio's kernel, per bratio branch.

usage: tails.py [args-variant ...]   (default: all variants). Writes tails.pkl (rows + per-variant values + branch).

Argument variants (what the T wrapper passes to K(a, b, x, y) with a = n/2, b = 1/2):
  yz     K(n/2, 1/2, y, z): one call, the caller's complement z = fl(tt/u)
  y1y    K(n/2, 1/2, y, fl(1 - y))
  split  t <= 1 (tt <= 1): K(1/2, n/2, z, fl(1 - z)), the BETA.DIST(z) cell; else K(n/2, 1/2, y, fl(1 - y))
  splitz t <= 1: K(n/2, 1/2, y, z); else K(n/2, 1/2, y, fl(1 - y))
  cut99  K(n/2, 1/2, y, z) for y > 0.99, else K(n/2, 1/2, y, fl(1 - y))
  t2     K(n/2, 1/2, y, z), but fl(1 - y) at df 2
  hyb    K(n/2, 1/2, y, z) for tt <= 1 or y > 0.99, else K(n/2, 1/2, y, fl(1 - y))
The tail L = P(T > |t|) is then (computed lower tail of I_y(n/2, 1/2))/2; T.DIST.2T = 2L = I_y.
Upper side compositions (T.DIST(t > 0, TRUE), T.DIST.RT(t < 0)) are scored separately in compose.py.
"""

from __future__ import annotations

import pickle
import sys
from collections import defaultdict

import tlib

VARIANTS = ("yz", "y1y", "split", "splitz", "cut99")


def kargs(variant: str, t: float, n: float):
    tt, u, y, z = tlib.targs(t, n)
    if variant == "yz":
        return (n / 2, 0.5, y, z), False
    if variant == "y1y":
        return (n / 2, 0.5, y, 1.0 - y), False
    if variant == "split":
        if tt <= 1.0:
            return (0.5, n / 2, z, 1.0 - z), True
        return (n / 2, 0.5, y, 1.0 - y), False
    if variant == "splitz":
        if tt <= 1.0:
            return (n / 2, 0.5, y, z), False
        return (n / 2, 0.5, y, 1.0 - y), False
    if variant == "cut99":  # statistics.md: exact-at-t for y > 0.99, rounded y below
        return (n / 2, 0.5, y, z if y > 0.99 else 1.0 - y), False
    if variant == "t2":  # K(n/2, 1/2, y, z), except df 2: K(1, 1/2, y, fl(1 - y)) (w5's df-2 closed forms)
        return (n / 2, 0.5, y, 1.0 - y if n == 2 else z), False
    if variant == "hyb":  # z in the centre (tt <= 1) and for y > 0.99, else fl(1 - y)
        return (n / 2, 0.5, y, z if (tt <= 1.0 or y > 0.99) else 1.0 - y), False
    raise ValueError(variant)


def tail_I(variant: str, t: float, n: float, memo: dict):
    """(I_y(n/2,1/2) as Excel's kernel gives it, branch)."""
    args, flipped = kargs(variant, t, n)
    if args not in memo:
        memo[args] = bx_kernel(*args)
    w, w1, br = memo[args]
    return (w1 if flipped else w), br


def bx_kernel(a, b, x, y):
    if x == 0.0:
        return 0.0, 1.0, "x=0"
    return tlib.bx.kernel(a, b, x, y, tlib.bx.BEST)


def value(kind: str, t: float, I: float):
    """Cell value from the kernel's I_y: L = I/2."""
    L = I / 2
    if kind == "2t":
        return I
    if (kind == "rt") == (t >= 0):
        return L
    return 1.0 - L  # upper side: compose.py tests the alternatives


def main():
    vs = sys.argv[1:] or list(VARIANTS)
    rows = tlib.load(("rt", "2t", "cdf"))
    memo: dict = {}
    vals = {}
    br = {}
    for v in vs:
        out, bb = [], []
        for r in rows:
            if r["t"] == 0.0:
                out.append(0.5 if r["kind"] != "2t" else 1.0)
                bb.append("t=0")
                continue
            I, b = tail_I(v, abs(r["t"]), r["n"], memo)
            out.append(value(r["kind"], r["t"], I))
            bb.append(b)
        vals[v], br[v] = out, bb
        print("done", v, file=sys.stderr)
    for r in rows:
        r["cr"] = tlib.exact_cell(r)
    tlib.save_exact()
    pickle.dump((rows, vals, br), open(tlib.HERE / "tails.pkl", "wb"))
    report(rows, vals, br)


def report(rows, vals, br, key_variant="yz"):
    g = defaultdict(list)
    for i, r in enumerate(rows):
        side = "upper" if (r["kind"] == "cdf") == (r["t"] > 0) and r["kind"] != "2t" else "L"
        g[(br[key_variant][i], side)].append(i)
    g[("ALL", "")] = list(range(len(rows)))
    names = list(vals)
    print(
        f"| branch ({key_variant}) | side | n | "
        + " | ".join(names)
        + " | CR | hold-out "
        + " | ".join(names)
        + " | CR |"
    )
    print("|---" * (2 * len(names) + 5) + "|")
    for k in sorted(g, key=str):
        idx = g[k]
        cells = []
        for v in names:
            n, ex, f, w = tlib.TALLY((vals[v][i], rows[i]["excel"]) for i in idx)
            cells.append(f"{ex}/{f}/{w}")
        n, ex, f, w = tlib.TALLY((rows[i]["cr"], rows[i]["excel"]) for i in idx)
        hidx = [i for i in idx if rows[i]["hold"]]
        hs = [tlib.TALLY((vals[v][i], rows[i]["excel"]) for i in hidx)[1] for v in names]
        hc = tlib.TALLY((rows[i]["cr"], rows[i]["excel"]) for i in hidx)[1]
        print(
            f"| {k[0]} | {k[1]} | {n} | "
            + " | ".join(cells)
            + f" | {ex}/{f}/{w} | "
            + " | ".join(f"{h}/{len(hidx)}" for h in hs)
            + f" | {hc}/{len(hidx)} |"
        )


if __name__ == "__main__":
    main()
