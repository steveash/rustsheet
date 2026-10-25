"""Inverse functions: how many visibly-off BETA.INV / F.INV / T.INV cells a crossing search on the round-2 model CDF
(r2-bratio BEST + r2-log1p) would fix. statistics.md "BETA.INV solves Excel's own CDF to the crossing" (155/188):
the estimate bisects over doubles to the adjacent pair (lo, hi) where the model CDF crosses p and scores hi, lo and
"either" against Excel. Standing cells with numeric single-call arguments only.
"""

import struct
from collections import Counter

import implib as lib

MODEL = "BEST+log1p"


def _ord(x: float) -> int:
    return struct.unpack("<q", struct.pack("<d", x))[0]


def _flt(i: int) -> float:
    return struct.unpack("<d", struct.pack("<q", i))[0]


def crossing(f, p: float, lo: float, hi: float, increasing: bool):
    """Adjacent doubles (a, b), a < b, with f(a) < p <= f(b) (increasing) or f(a) > p >= f(b) (decreasing)."""
    i, j = _ord(lo), _ord(hi)
    while j - i > 1:
        m = (i + j) // 2
        v = f(_flt(m))
        if (v >= p) if increasing else (v <= p):
            j = m
        else:
            i = m
    return _flt(i), _flt(j)


def setup(fn: str, args: tuple):
    """(f, p, lo, hi, increasing, post) for one inverse cell, or None."""
    tr = lib._tr
    if fn in ("BETA.INV", "BETAINV"):
        p, a, b = args[:3]
        A = args[3] if len(args) > 3 else 0.0
        B = args[4] if len(args) > 4 else 1.0
        if not (0 < p < 1 and a > 0 and b > 0 and A < B):
            return None
        return (("BETA.DIST", lambda x: (x, a, b, True)), p, 5e-324, 1.0, True, lambda x: A + x * (B - A))
    if fn in ("F.INV", "F.INV.RT", "FINV"):
        p, d1, d2 = args[:3]
        if not (0 < p < 1 and 1 <= tr(d1) < 1e10 and 1 <= tr(d2) < 1e10):
            return None
        g = "F.DIST" if fn == "F.INV" else "F.DIST.RT"
        mk = (lambda x: (x, d1, d2, True)) if g == "F.DIST" else (lambda x: (x, d1, d2))
        return ((g, mk), p, 5e-324, 1e300, fn == "F.INV", lambda x: x)
    if fn == "T.INV":
        p, n = args[:2]
        if not (0 < p < 1 and 1 <= tr(n) < 1e10):
            return None
        if p < 0.5:
            return (("T.DIST.RT", lambda t: (t, n)), p, 1e-300, 1e300, False, lambda t: -t)
        return (("T.DIST", lambda t: (t, n, True)), p, 1e-300, 1e300, True, lambda t: t)
    if fn in ("T.INV.2T", "TINV"):
        p, n = args[:2]
        if not (0 < p <= 1 and 1 <= tr(n) < 1e10):
            return None
        return (("T.DIST.2T", lambda t: (t, n)), p, 1e-300, 1e300, False, lambda t: t)
    return None


def main() -> None:
    import bratio_excel as bx

    name, cfg, l1p = next(c for c in lib.configs() if c[0] == MODEL)
    lib.set_log1p(l1p)
    fails, _ = lib.failing_cells()
    eng = {(c["case"], c["key"]): c for c in fails}
    t = Counter()
    per_fn = Counter()
    for r in lib.all_cells():
        if lib.FAMILY.get(r["fn"]) != "inv" or "/t2464-" in r["case"]:
            continue
        s = setup(r["fn"], r["args"])
        if s is None:
            continue
        (g, mk), p, lo, hi, inc, post = s

        def f(x, g=g, mk=mk):
            pr = lib.problem(g, mk(x))
            if pr[0] != "cdf":
                return 0.0 if x <= 1 else 1.0
            w, w1, _ = bx.kernel(pr[1], pr[2], pr[3], pr[4], cfg)
            return lib.finish_cdf(pr[6], w if pr[5] == "lower" else w1)

        a, b = crossing(f, p, lo, hi, inc)
        if bx._GL_MISSING:
            bx.fetch_missing()
            a, b = crossing(f, p, lo, hi, inc)
        e = r["excel"]
        c = eng.get((r["case"], r["key"]))
        g_ = c["engine"] if c else e
        gv = isinstance(g_, float) and lib.within15(e, g_)
        ha, hb = lib.within15(e, post(a)), lib.within15(e, post(b))
        t["n"] += 1
        t["eng_vis"] += not gv
        t["fix_hi"] += (not gv) and hb
        t["fix_either"] += (not gv) and (ha or hb)
        t["broke_hi"] += gv and not hb
        t["exact_either"] += e in (post(a), post(b))
        per_fn[(r["fn"], "n")] += 1
        per_fn[(r["fn"], "eng_vis")] += not gv
        per_fn[(r["fn"], "fix_either")] += (not gv) and (ha or hb)
        per_fn[(r["fn"], "broke_hi")] += gv and not hb
    print(f"crossing search on {MODEL}, standing pre-T2464 inverse cells with numeric args:", dict(t))
    for fn in sorted({k[0] for k in per_fn}):
        print(
            f"  {fn:10s} n {per_fn[(fn, 'n')]:4d} eng_vis {per_fn[(fn, 'eng_vis')]:4d} "
            f"fixed(either side) {per_fn[(fn, 'fix_either')]:4d} broke(hi) {per_fn[(fn, 'broke_hi')]:4d}"
        )


if __name__ == "__main__":
    main()
