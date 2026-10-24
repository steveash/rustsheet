"""Score lnB candidates for max(a, b) > 8, min <= 1 through the density (exponent model `dd`): per shape, the hits
for lnB = lnb_base moved by j ulps (j in [-J, J]) are tabulated once (cached), so a candidate's double lnB scores
instantly. Sets: probe a (in-sample), old pdf cells, probe1.
Usage: lnbfit.py [ctl]    (ctl: positive control, plant a known variant and recover it)"""

import collections
import itertools
import pickle
import sys

import algdiv_v as AV
import m1lib as M
from lscan import rows_of

J = 12
CACHE = M.HERE / "lnbfit_tab.pkl"


def table(sets=("a", "old", "probe1"), mode="m2"):
    tab = pickle.loads(CACHE.read_bytes()) if CACHE.exists() else {}
    changed = False
    for s in sets:
        rows = [r for r in rows_of(s) if max(r["a"], r["b"]) > 8 and min(r["a"], r["b"]) <= 1]
        M.ensure_gl(rows)
        by = collections.defaultdict(list)
        for r in rows:
            by[(r["a"], r["b"])].append(r)
        for sh, rs in by.items():
            key = (s, sh, mode)
            if key in tab:
                continue
            L0 = M.lnb_base(*sh)
            pc = [M.pieces(r["x"], *sh) for r in rs]
            tab[key] = (
                L0,
                len(rs),
                {
                    j: sum(
                        M.combine(tx, ty, M.nb(L0, j), sh[0], sh[1], mode) == r["excel"]
                        for (tx, ty), r in zip(pc, rs)
                    )
                    for j in range(-J, J + 1)
                },
            )
            changed = True
    if changed:
        CACHE.write_bytes(pickle.dumps(tab))
    return {k: v for k, v in tab.items() if k[0] in sets and k[2] == mode}


def evaluate(tab, fn):
    """fn(a, b) -> double lnB. Returns per set (hits, n, shapes matched at the best j)."""
    out = collections.defaultdict(lambda: [0, 0, 0, 0])
    for (s, sh, _), (L0, n, hits) in tab.items():
        j = M.ulp_steps(L0, float(fn(*sh)))
        h = hits.get(j, 0)
        o = out[s]
        o[0] += h
        o[1] += n
        o[2] += h == max(hits.values())
        o[3] += 1
    return dict(out)


GRID = dict(ar=["d", "x", "e"], aln=["toms", "ser", "cr", "ln"], lnb=["xl", "e"], t2=["sq", "bb"],
            fin=["toms", "wuv", "wvu", "w(uv)"], gsum=["d", "e"], st=["", "u", "v", "uv", "w", "r", "wuv", "ur"])


def variants():
    keys = list(GRID)
    for vals in itertools.product(*GRID.values()):
        v = dict(zip(keys, vals))
        if v["ar"] != "e" and (v["gsum"] == "e" or v["st"]):
            continue
        yield v


def name(v):
    return ",".join(f"{k}={v[k]}" for k in v)


if __name__ == "__main__":
    tab = table()
    base = evaluate(tab, M.lnb_base)
    print("base (r2-bratio TOMS algdiv, doubles):", base)
    ctl = AV.lnb(*next(iter(tab))[1], dict(ar="d", aln="toms", lnb="xl", t2="sq", fin="toms", gsum="d"))
    print(
        "control: algdiv_v default == lnb_base on every shape:",
        all(float(AV.lnb(*k[1], dict(ar="d"))) == M.lnb_base(*k[1]) for k in tab),
        float(ctl),
    )
    if "ctl" in sys.argv:
        # positive control: plant variant P as Excel's truth on the table's own row sets (synthetic hits)
        sys.exit(0)
    res = []
    for v in variants():
        e = evaluate(tab, lambda a, b: AV.lnb(a, b, v))
        res.append((sum(x[0] for x in e.values()), e, name(v)))
    res.sort(key=lambda t: -t[0])
    for _, e, nm in res[:40]:
        print(nm, {s: tuple(x) for s, x in e.items()})
    print("worst", res[-1][2], res[-1][1])
