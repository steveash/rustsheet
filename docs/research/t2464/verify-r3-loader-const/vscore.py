"""verify-r3-loader-const: re-score M3 / M4 (independent implementation, vmodel.py) on every a, b > 1 pdf cell.

Usage: vscore.py [--worker] [--refresh]
  --worker  also run the worker's m1_model.density (M3, M4) and report cell-by-cell agreement with vmodel
Groups: the worker's four probes; r2-era data (cases without the t2513 prefix); the other round-3 workers' probes
(never read by r3-loader-const); this verifier's own hold-out (t2513-verify-r3-loader-const-*).
Counts: bit-exact / within 4 ulp of Excel / within half a unit of Excel's 15th significant digit.
"""

from __future__ import annotations

import math
import pickle
import sys
from collections import Counter, defaultdict

import vdata
import vmodel as VM

HERE = vdata.HERE


def ulpdist(m: float, e: float) -> float:
    if m == e:
        return 0.0
    if not math.isfinite(m):
        return math.inf
    return abs(m - e) / math.ulp(e)


def w15(m: float, e: float) -> bool:
    if m == e:
        return True
    if not math.isfinite(m) or e == 0:
        return False
    return abs(m - e) <= 0.5 * 10.0 ** (math.floor(math.log10(abs(e))) - 14)


def tally(pairs):
    pairs = list(pairs)
    return (
        len(pairs),
        sum(m == e for m, e in pairs),
        sum(ulpdist(m, e) <= 4 for m, e in pairs),
        sum(w15(m, e) for m, e in pairs),
    )


def fmt(t):
    n, ex, w4, w = t
    return f"{ex:6d} / {w4:6d} / {w:6d} of {n:6d}"


def group(case: str) -> str:
    if case.startswith("t2513-r3-loader-const-hold"):
        return "worker-hold"
    if case.startswith("t2513-r3-loader-const-"):
        return "worker-insample:" + case.rsplit("-", 1)[1]
    if case.startswith("t2513-verify-r3-loader-const"):
        return "verifier-hold"
    if case.startswith("t2513-"):
        return "other-r3-probes"
    return "r2-era"


def dedup(rows):
    """Keep the first row per (x, a, b); report duplicates and Excel disagreements among them."""
    seen, out, dup, conflict = {}, [], 0, 0
    for r in rows:
        key = (r["x"], r["a"], r["b"])
        if key in seen:
            dup += 1
            conflict += seen[key]["excel"] != r["excel"]
            continue
        seen[key] = r
        out.append(r)
    return out, dup, conflict


_PC = HERE / "preds.pkl"


def predict(rows, refresh=False):
    cache = pickle.loads(_PC.read_bytes()) if _PC.exists() and not refresh else {}
    names = [*VM.MODELS, "CR", "log2"]
    todo = [r for r in rows if not all(nm in cache.get((r["x"], r["a"], r["b"]), {}) for nm in names)]
    if todo:
        need = []
        for r in todo:
            need += VM.ln_needs(r["x"], r["a"], r["b"])
        VM.ln_prefetch(need)
        for i, r in enumerate(todo):
            key = (r["x"], r["a"], r["b"])
            p = cache.setdefault(key, {})
            for nm, o in VM.MODELS.items():
                if nm not in p:
                    p[nm] = VM.density(*key, **o)
            if "CR" not in p:
                p["CR"] = VM.exact_pdf(*key)
            p["log2"] = all(VM.is_log(kv, M) for kv, M in VM.sides(*key))
            if i % 5000 == 4999:
                _PC.write_bytes(pickle.dumps(cache))
        _PC.write_bytes(pickle.dumps(cache))
    return cache


def controls():
    """EXP V6 (mpf model) against the engine harness on doubles; LN2PI constant against w6's M_LN_2PI."""
    import random

    rng = random.Random(7)
    xs = [rng.uniform(-30, 5) for _ in range(3000)]
    h = VM.excelmath.excel("EXP", xs)
    m = [VM.W1.exp_v6(VM.mpf(v)) for v in xs]
    print(f"control EXP V6 model vs harness on 3000 doubles: {sum(a == b for a, b in zip(h, m))} equal")
    sys.path.append(str(VM.T / "w6-literature"))
    import h2_loader

    print(f"control LN2PI == w6 M_LN_2PI: {float(VM.LN2PI) == h2_loader.M_LN_2PI}")


def main():
    refresh = "--refresh" in sys.argv
    controls()
    rows, stats = vdata.load_all(refresh)
    unparsed = {k: v for k, v in stats.items() if v[0] != v[1]}
    print(f"rows {len(rows)}; cases with unparsed pdf formulas: {unparsed}")
    rows, dup, conflict = dedup(rows)
    print(f"after dedup on (x,a,b): {len(rows)} ({dup} duplicates, {conflict} with a different Excel value)")
    P = predict(rows, refresh)
    names = list(VM.MODELS) + ["CR"]
    by = defaultdict(list)
    for r in rows:
        g = group(r["case"])
        by[g].append(r)
        if g.startswith("worker-insample"):
            by["worker-insample:all"].append(r)
        if g in ("r2-era", "other-r3-probes", "verifier-hold"):
            by["not-seen-by-worker:all"].append(r)
        if g == "r2-era" and r["case"] != "t2464-r2-log1p-hix":
            by["r2-like (r2-era minus log1p-hix)"].append(r)
    for g in sorted(by):
        rs = by[g]
        print(
            f"== {g}  (n {len(rs)}, both sides log form {sum(P[(r['x'], r['a'], r['b'])]['log2'] for r in rs)})"
        )
        for nm in names:
            print(f"  {nm:8s} {fmt(tally((P[(r['x'], r['a'], r['b'])][nm], r['excel']) for r in rs))}")
        if g in ("worker-hold", "other-r3-probes", "verifier-hold", "r2-like (r2-era minus log1p-hix)"):
            for sub, keep in (("both-log", True), ("a series side", False)):
                ss = [r for r in rs if P[(r["x"], r["a"], r["b"])]["log2"] == keep]
                for nm in ("M3", "M4", "Mexact", "CR"):
                    print(
                        f"    [{sub}] {nm:8s} {fmt(tally((P[(r['x'], r['a'], r['b'])][nm], r['excel']) for r in ss))}"
                    )
    if "--worker" in sys.argv:
        worker_agreement(by)


def worker_agreement(by):
    """Cell-by-cell agreement of vmodel with the worker's m1_model.density (imported only for this check)."""
    sys.path.append(str(VM.T / "r3-loader-const"))
    import m1_model as WM

    P = predict([])
    for g in ("worker-hold", "r2-like (r2-era minus log1p-hix)"):
        rs = by[g]
        for nm in ("M3", "M4"):
            agree = Counter()
            for r in rs:
                key = (r["x"], r["a"], r["b"])
                w = WM.density(r["x"], r["a"], r["b"], WM.MODELS[nm])
                agree[w == P[key][nm]] += 1
                agree["worker_exact"] += w == r["excel"]
            print(
                f"worker {g} {nm}: agree {agree[True]} disagree {agree[False]}; worker exact {agree['worker_exact']}"
            )


if __name__ == "__main__":
    main()
