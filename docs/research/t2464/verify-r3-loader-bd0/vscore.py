"""Verifier's scorer for r3-loader-bd0: one free constant C per block (or per shape) with chain Ce,
out = EXP_V6(C + X) in extended precision.

Per cell, the admissible extended exponents {E : EXP_V6(E) == excel} form an interval found by bisection around
ln(excel) (monotone map); the cell's admissible C is that interval minus the model's X.  C is the centre of the
deepest overlap on the fit half; every scored cell is then predicted directly (EXP_V6(C + X)) and compared.

Usage: vscore.py CASE_SUFFIX [...]  [--split par|md5x|half] [--group block|shape]
  splits: par = cells with odd index in the block are scored (C from the even ones), and the reverse fold;
          half = C from the first half of each block, scored on the second half (and reverse).
  Models: CR (correctly rounded, no fit), null (exact ln pdf + C), r2-x87, r3, r3 without each knob.
"""
from __future__ import annotations

import math
import pickle
import sys
from collections import defaultdict
from pathlib import Path

import mpmath

import vdata as V
import vmodel as M

HERE = Path(__file__).parent
U = M.LD(2.0) ** -53
MODELS = {
    "r2-x87": {"v": "e", "ph": 0, "ld": "e"},
    "r3": {"v": "x", "ph": 1, "ld": "x"},
    "r3-ph": {"v": "x", "ph": 0, "ld": "x"},
    "r3-ld": {"v": "x", "ph": 1, "ld": "e"},
    "r3-v": {"v": "e", "ph": 1, "ld": "x"},
}
EXPLORE = {  # verifier's exploratory log-form variants (selected after seeing the verifier hold-out)
    "r3+xx": {"v": "x", "ph": 1, "ld": "xx"},
    "r3+R": {"v": "x", "ph": 1, "ld": "R"},
    "r3+Xs": {"v": "x", "ph": 1, "ld": "x", "Xs": 1},
    "r3+xxXs": {"v": "x", "ph": 1, "ld": "xx", "Xs": 1},
}
if "--explore" in sys.argv:
    MODELS.update(EXPLORE)
_IVC = HERE / "einterval_cache.pkl"
_IV: dict = pickle.loads(_IVC.read_bytes()) if _IVC.exists() else {}


def within15(m: float, e: float) -> bool:
    if m == e:
        return True
    if e == 0 or not math.isfinite(m):
        return False
    return abs(m - e) <= 0.5 * 10.0 ** (math.floor(math.log10(abs(e))) - 14)


def ulp_ok(m: float, e: float, n: int = 4) -> bool:
    return abs(m - e) <= n * math.ulp(e)


def e_interval(y: float, W: float = 64.0, tol: float = 1e-4):
    """(E0, lo, hi): admissible E = E0 + u 2^-53 for u in [lo, hi]; None if EXP_V6 never returns y."""
    r = _IV.get(y)
    if r is not None:
        return r
    E0 = M.ld_of(mpmath.log(y))

    def F(u):
        return M.exp_v6(E0 + M.LD(u) * U)

    out = None
    if F(-W) <= y <= F(W):
        lo, hi = -W, W
        while hi - lo > tol:
            m = (lo + hi) / 2
            lo, hi = (lo, m) if F(m) >= y else (m, hi)
        umin = hi
        lo, hi = umin, W
        while hi - lo > tol:
            m = (lo + hi) / 2
            lo, hi = (m, hi) if F(m) <= y else (lo, m)
        if F(umin) == y:
            out = (E0, umin, lo)
    _IV[y] = out
    return out


def deepest(ivs):
    ev = sorted([(lo, 0) for lo, hi in ivs] + [(hi, 1) for lo, hi in ivs])
    c = best = 0
    pos = 0.0
    for i, (p, t) in enumerate(ev):
        if t == 0:
            c += 1
            if c > best:
                best, pos = c, (p + ev[i + 1][0]) / 2
        else:
            c -= 1
    return pos


def folds(idx, split):
    n = len(idx)
    if split == "par":
        a = [i for j, i in enumerate(idx) if j % 2 == 0]
        b = [i for j, i in enumerate(idx) if j % 2 == 1]
    else:  # half
        a, b = idx[: n // 2], idx[n // 2:]
    return [(a, b), (b, a)]


def run(suffixes, split="par", group="block"):
    rows = []
    for s in suffixes:
        rows += V.load_case(f"t2513-{s}" if s.startswith("verify") else f"t2513-r3-loader-bd0-{s}")
    M.LN.need([r for row in rows for r in M.ratios_needed(row["x"], row["a"], row["b"])])
    X = {name: [M.xpart(r["x"], r["a"], r["b"], o) for r in rows] for name, o in MODELS.items()}
    X["null"] = [(M.exact_logpdf(r["x"], r["a"], r["b"]), X["r3"][i][1]) for i, r in enumerate(rows)]
    br = [X["r3"][i][1] for i in range(len(rows))]
    ivs = [e_interval(r["excel"]) for r in rows]
    _IVC.write_bytes(pickle.dumps(_IV))
    grp: dict = defaultdict(list)
    for blk, idx in V.blocks(rows).items():
        key = blk if group == "block" else (rows[idx[0]]["case"], rows[idx[0]]["a"], rows[idx[0]]["b"])
        grp[key].append(idx)
    res = defaultdict(lambda: [0, 0, 0, 0])
    per_block = defaultdict(dict)
    for name in ["CR", "null"] + list(MODELS):
        for key, idxs in grp.items():
            for f in range(2):
                fit = [i for idx in idxs for i in folds(idx, split)[f][0]]
                sc = [i for idx in idxs for i in folds(idx, split)[f][1]]
                if name == "CR":
                    pred = {i: M.exact_pdf(rows[i]["x"], rows[i]["a"], rows[i]["b"]) for i in sc}
                else:
                    Xs = X[name]
                    Cref = M.LD(ivs[fit[0]][0]) - Xs[fit[0]][0] if ivs[fit[0]] else -Xs[fit[0]][0]
                    cint = []
                    for i in fit:
                        iv = ivs[i]
                        if iv is None:
                            continue
                        base = float((iv[0] - Xs[i][0] - Cref) / U)
                        cint.append((base + iv[1], base + iv[2]))
                    C = Cref + M.LD(deepest(cint)) * U
                    pred = {i: M.exp_v6(C + Xs[i][0]) for i in sc}
                for i in sc:
                    e = rows[i]["excel"]
                    for t in (res[(name, "ALL")], res[(name, br[i])]):
                        t[0] += 1
                        t[1] += pred[i] == e
                        t[2] += ulp_ok(pred[i], e)
                        t[3] += within15(pred[i], e)
                    pb = per_block[(name, rows[i]["blk"])]
                    pb["n"] = pb.get("n", 0) + 1
                    pb["x"] = pb.get("x", 0) + (pred[i] == e)
    return rows, res, per_block, br


def main(argv):
    split, group = "par", "block"
    if "--split" in argv:
        split = argv[argv.index("--split") + 1]
    if "--group" in argv:
        group = argv[argv.index("--group") + 1]
    sufs = [a for a in argv if not a.startswith("--") and a not in (split, group)]
    rows, res, pb, br = run(sufs, split, group)
    brs = sorted({b for _, b in res if b != "ALL"})
    print(f"== {sufs} split={split} group={group} (both folds: every cell scored once) "
          f"exact/w4ulp/w15 of n")
    for name in ["CR", "null"] + list(MODELS):
        parts = [f"{b}: {res[(name, b)][1]}/{res[(name, b)][2]}/{res[(name, b)][3]} of {res[(name, b)][0]}"
                 for b in ["ALL"] + brs]
        print(f"  {name:8s} " + "   ".join(parts))
    if "--blocks" in argv:
        for blk in sorted({r["blk"] for r in rows}, key=lambda s: (s.split("#")[0], int(s.split("#")[1]))):
            i0 = next(i for i, r in enumerate(rows) if r["blk"] == blk)
            r = rows[i0]
            print(f"  {blk:8s} ({r['a']:g},{r['b']:g}) x={r['x']:.6f} {br[i0]}  " +
                  " ".join(f"{n}={pb[(n, blk)]['x']}/{pb[(n, blk)]['n']}" for n in ["CR", "null", "r2-x87", "r3"]))


if __name__ == "__main__":
    main(sys.argv[1:])
