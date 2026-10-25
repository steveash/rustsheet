"""Exhaustive x87 storage search for algdiv: ar = "e" with every subset of the named intermediates stored
(h c x d s w u v r), alnrel in {toms, ser, cr, ln}, log b in {xl, e}, final order in {toms, wuv, wvu}, and the GL + alg
sum stored or extended. Scored through lnbfit's per-shape tables on probe a and probe1 (in-sample).
A positive control plants one variant's lnB as the truth and checks that the search ranks it first.
Usage: algsearch.py [ctl]"""
import itertools
import sys

import algdiv_v as AV
import lnbfit as LF
import m1lib as M

NAMES = "dwuvrs"  # stores of h, c, x left out to keep the run short


def grid():
    for k in range(len(NAMES) + 1):
        for st in itertools.combinations(NAMES, k):
            for aln in ("toms", "ser", "cr", "ln"):
                for lnb in ("xl", "e"):
                    for fin in ("toms", "wuv", "wvu"):
                        for gsum in ("d", "e"):
                            yield dict(ar="e", st="".join(st), aln=aln, lnb=lnb, fin=fin, gsum=gsum)


def jvec(v, shapes):
    return [M.ulp_steps(L0, float(AV.lnb(*sh, v))) for sh, L0 in shapes]


if __name__ == "__main__":
    # min(a, b) < 1 only: min = 1 is -LN(max) (final_score.py), not algdiv
    tab = {k: v for k, v in LF.table().items() if k[0] in ("a", "probe1", "old") and min(k[1]) < 1.0}
    keys = sorted(tab)
    shapes = [(k[1], tab[k][0]) for k in keys]
    ctl = "ctl" in sys.argv
    planted = dict(ar="e", st="wuv", aln="toms", lnb="e", fin="toms", gsum="d")
    pj = jvec(planted, shapes) if ctl else None
    res = []
    for v in grid():
        js = jvec(v, shapes)
        if ctl:
            sc = sum(a == b for a, b in zip(js, pj))
        else:
            sc = sum(tab[k][2].get(j, 0) for k, j in zip(keys, js))
        res.append((sc, LF.name(v)))
    res.sort(key=lambda t: -t[0])
    if ctl:
        top = [nm for sc, nm in res if sc == res[0][0]]
        print("planted", LF.name(planted), "recovered:", LF.name(planted) in top, "ties", len(top), "of", len(res))
    else:
        base = sum(tab[k][2][0] for k in keys)
        best = sum(max(tab[k][2].values()) for k in keys)
        print("variants", len(res), "shapes", len(keys), "base (TOMS double)", base, "per-shape best", best)
        for sc, nm in res[:25]:
            print(sc, nm)
