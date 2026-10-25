"""s4 chain with h scanned on a fine grid (h treated as an extended constant: steps of ulp(h)/8 over +-48 ulps),
i.e. g = fl(lc - h) with h not necessarily a double.  Usage: s5_hfine.py BLOCK knobs [lcmode]"""
import math
import sys
import mpmath
import dlib as L
import s4_chain as C4

rows = C4.rows


def run(blk, spec, lcmode="x", NH=48, sub=8, quiet=False):
    pre = C4.prep(blk, spec, lcmode)
    idx, a, b, k, n, lc = pre
    L5 = mpmath.mpf(L.LN(a + b - 1))
    h0 = float(C4.lf_ideal(k, n) / 2)
    u = mpmath.mpf(math.ulp(h0)) / sub
    ev = [i for i in idx if not rows[i]["hold"]]
    od = [i for i in idx if rows[i]["hold"]]
    cnt = lambda h, sel: sum(C4.chain(lc[i], h, L5) == rows[i]["excel"] for i in sel)  # noqa: E731
    res = [(cnt(h0 + j * u, ev), j) for j in range(-NH * sub, NH * sub + 1)]
    best = max(res)
    hb = h0 + best[1] * u
    ho = cnt(hb, od)
    if not quiet:
        top = sorted(res, reverse=True)[:4]
        print(f"{blk:10s} ({a:g},{b:g}) [{spec}] lc={lcmode} h=h0{best[1] / sub:+.3f}ulp even {best[0]}/{len(ev)}"
              f" hold {ho}/{len(od)}  top {[(c, j / sub) for c, j in top]}", flush=True)
    return hb, best, ho, len(od)


if __name__ == "__main__":
    for lm in sys.argv[3:] or ["x"]:
        run(sys.argv[1], sys.argv[2], lm)
