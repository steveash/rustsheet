"""T-3569 r5-algdiv: x87 storage variants of TOMS algdiv (b0 >= 8) scored per shape against r4-algdiv's pinned lnB.

A shape is pinned when r4-algdiv's scan found one j (ulps from lnb_base) that reproduces all of its rows
(`scan_<probe>.json`: best == n and a single tie). A variant scores one point per pinned shape whose lnB equals
the pinned lnB exactly. The grid is r3-m1-residual's `algdiv_v` switches: ar = "e" with every subset of the nine
named stores (h c x d s w u v r), plus ar = "d" and ar = "x" (all ops double / all ops x87 stored), each with
alnrel in {toms, ser, cr, ln}, log b in {xl, e}, final order in {toms, wuv, wvu, w(uv)}, t2 in {sq, bb} and the
GL + algdiv sum stored or extended.

Usage: score.py ctl|a|hold [--top N]
  ctl   positive control on probe a's shapes: plant one variant's lnB as the pins, check it ranks first
  a     in-sample scoring (probe a)
  hold  score the frozen top variants of probe a on the hold-out (run once, after `a`)
Writes score_<mode>.json (all variant scores) and prints the top of the ranking."""

from __future__ import annotations

import itertools
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-m1-residual"))
import algdiv_v as AV  # noqa: E402
import m1lib as M  # noqa: E402

NAMES = "hcxdswuvr"


def grid():
    arith = [dict(ar="d"), dict(ar="x")]
    for k in range(len(NAMES) + 1):
        for st in itertools.combinations(NAMES, k):
            arith.append(dict(ar="e", st="".join(st)))
    for base in arith:
        for aln in ("toms", "ser", "cr", "ln"):
            for lnb in ("xl", "e"):
                for fin in ("toms", "wuv", "wvu", "w(uv)"):
                    for t2 in ("sq", "bb"):
                        for gsum in ("d", "e"):
                            yield dict(base, aln=aln, lnb=lnb, fin=fin, t2=t2, gsum=gsum)


def name(v: dict) -> str:
    return " ".join(f"{k}={v[k]}" for k in ("ar", "st", "aln", "lnb", "fin", "t2", "gsum") if k in v)


def pins(probe: str):
    """[(a, b, pinned lnB)] for the uniquely pinned shapes of r4-algdiv's probe."""
    rows = json.loads((HERE.parent / "r4-algdiv" / f"scan_{probe}.json").read_text())
    out = []
    for a, b, n, _h0, jb, best, ties, *_ in rows:
        if best == n and len(ties) == 1:
            out.append((a, b, M.nb(M.lnb_base(a, b), jb)))
    return out


def lnb_of(v: dict, a: float, b: float) -> float:
    lo, hi = min(a, b), max(a, b)
    return float(AV.lnb(lo, hi, v))


def score(v: dict, shapes) -> int:
    return sum(lnb_of(v, a, b) == L for a, b, L in shapes)


def _score_one(args):
    v, shapes = args
    return score(v, shapes), name(v)


def run(shapes, variants):
    from multiprocessing import Pool

    with Pool(int(__import__("os").environ.get("JOBS", "3"))) as pool:
        res = pool.map(_score_one, ((v, shapes) for v in variants), chunksize=64)
    res.sort(key=lambda t: -t[0])
    return res


def main():
    mode = sys.argv[1]
    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 25
    if mode == "ctl":
        shapes = pins("a")
        planted = dict(ar="e", st="dwu", aln="toms", lnb="e", fin="toms", t2="sq", gsum="d")
        fake = [(a, b, lnb_of(planted, a, b)) for a, b, _ in shapes]
        # The control grid keeps every store subset and fin, lnb; alnrel, t2 and gsum at the planted value.
        ctl_grid = [v for v in grid() if v["aln"] == "toms" and v["t2"] == "sq" and v["gsum"] == "d"]
        res = run(fake, ctl_grid)
        best = res[0][0]
        tied = [nm for sc, nm in res if sc == best]
        print(f"control: planted {name(planted)}; best score {best} of {len(fake)}; planted ranked first: "
              f"{name(planted) in tied}; ties at the top {len(tied)} of {len(res)}")
        return
    if mode == "a":
        shapes = pins("a")
        res = run(shapes, grid())
    elif mode == "hold":
        frozen = json.loads((HERE / "score_a.json").read_text())
        keep = {nm for sc, nm in frozen["ranking"][:top]}
        shapes = pins("hold")
        res = run(shapes, [v for v in grid() if name(v) in keep])
    else:
        raise SystemExit(__doc__)
    tomsd = score(dict(ar="d", aln="toms", lnb="xl", fin="toms", t2="sq", gsum="d"), shapes)
    print(f"{mode}: pinned shapes {len(shapes)}; variants {len(res)}; TOMS-d {tomsd}")
    for sc, nm in res[:top]:
        print(sc, nm)
    hist: dict[int, int] = {}
    for sc, _ in res:
        hist[sc] = hist.get(sc, 0) + 1
    print("score histogram (score: variants):", dict(sorted(hist.items(), reverse=True)[:12]))
    # Only the top tier is kept (the full ranking is several MB): every variant at the best score.
    tier = [r for r in res if r[0] == res[0][0]]
    (HERE / f"score_{mode}.json").write_text(
        json.dumps(dict(shapes=len(shapes), tomsd=tomsd, histogram=hist, ranking=tier)) + "\n"
    )


if __name__ == "__main__":
    main()
