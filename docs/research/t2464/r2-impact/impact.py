"""Steps 3-4: score the round-1/round-2 models cell by cell against Excel and the current engine, on every standing
eval cell of the incomplete-beta family that maps onto the kernel (cdf) or the beta density (pdf).

For each model: visibly-off cells fixed (engine visibly off -> model within 15 digits), cells made visibly off
(engine within 15 digits -> model not), bit-exact cells lost (engine bit-exact -> model not; in gated suites these
are baseline regressions), and the model's own exact / 4-ulp / 15-digit counts. Split by population, by the bratio
branch the cell takes (BEST's dispatch), by function and by md5 hold-out half.

Usage: impact.py [--fast]   (writes impact.pkl, a git-ignored cache of per-cell model values; prints the tables)
"""

import hashlib
import sys
from collections import Counter, defaultdict

import implib as lib

HYBRIDS = ("ENG+closed", "ENG+M1pdf", "ENG+M1pdf+log1p", "ENG+closed+M1pdf+log1p")


def population(case: str) -> str:
    if "/t2464-w2f-" in case:
        return "w2f-probes"
    if "/t2464-r2-" in case:
        return "r2-probes"
    return "pre-T2464"


def hold(case: str, key: str) -> bool:
    return int(hashlib.md5(f"{case}|{key}".encode()).hexdigest(), 16) % 2 == 1


def build():
    fails, _ = lib.failing_cells()
    eng = {(c["case"], c["key"]): c for c in fails}
    cells = []
    for r in lib.all_cells():
        pop = population(r["case"])
        if pop == "r2-probes":
            continue
        p = lib.problem(r["fn"], r["args"])
        if p[0] == "none":
            continue
        k = (r["case"], r["key"])
        f = eng.get(k)
        engine = f["engine"] if f else r["excel"]
        if not isinstance(engine, float):
            engine = float("nan")
        cells.append(
            dict(
                k=k,
                fn=r["fn"],
                case=r["case"],
                pop=pop,
                prob=p,
                excel=r["excel"],
                engine=engine,
                gated=r["case"].split("/")[0] not in lib.UNGATED,
                hold=hold(*k),
            )
        )
    probs = [(c["k"], c["prob"]) for c in cells]
    models = {}
    for name, cfg, l1p in lib.configs():
        print("model", name, file=sys.stderr)
        models[name] = lib.evaluate(probs, cfg, l1p)
    return cells, models


def hybrid(c, models, name):
    """The current engine with one round-1 piece dropped in where that piece applies."""
    v, br = models["BEST+log1p" if "log1p" in name else "BEST"][c["k"]]
    use = False
    if "closed" in name and br.startswith("closed"):
        use = True
    if "M1pdf" in name and br == "pdf:M1":
        use = True
    return v if use else c["engine"]


def tallies(cells, value):
    t = Counter()
    for c in cells:
        e, g, m = c["excel"], c["engine"], value(c)
        g15, m15 = lib.within15(e, g), lib.within15(e, m)
        t["n"] += 1
        t["eng_vis"] += not g15
        t["fixed"] += (not g15) and m15
        t["broke15"] += g15 and not m15
        t["lost_exact"] += (g == e) and (m != e)
        t["lost_exact_gated"] += (g == e) and (m != e) and c["gated"]
        t["gain_exact"] += (g != e) and (m == e)
        t["ex"] += m == e
        t["u4"] += lib.ulps(m, e) <= 4
        t["w15"] += m15
    t["net_vis"] = t["fixed"] - t["broke15"]
    return t


COLS = (
    "n",
    "eng_vis",
    "fixed",
    "broke15",
    "net_vis",
    "ex",
    "u4",
    "w15",
    "gain_exact",
    "lost_exact",
    "lost_exact_gated",
)


def show(title, cells, models, names):
    print(f"\n== {title}")
    print("  " + f"{'model':24s}" + "".join(f"{c:>10s}" for c in COLS))
    eng = tallies(cells, lambda c: c["engine"])
    print("  " + f"{'ENGINE (now)':24s}" + "".join(f"{eng[c]:10d}" for c in COLS))
    for nm in names:
        if nm in HYBRIDS:
            t = tallies(cells, lambda c, nm=nm: hybrid(c, models, nm))
        else:
            t = tallies(cells, lambda c, nm=nm: models[nm][c["k"]][0])
        print("  " + f"{nm:24s}" + "".join(f"{t[c]:10d}" for c in COLS))


def branch_of(c, models):
    br = models["BEST"][c["k"]][1].replace("/sw", "")
    if br == "pdf:CR-standin":
        a, b = c["prob"][2], c["prob"][3]
        br = "pdf:min>1,max>=8" if max(a, b) >= 8 else "pdf:min>1,max<8"
    if br == "pdf:M1":
        a, b = c["prob"][2], c["prob"][3]
        br = "pdf:M1 b=1" if b == 1 else "pdf:M1 min<=1"
    return br


def main() -> None:
    cells, models = lib.cache("impact", build)
    all_names = [n for n, _, _ in lib.configs()] + list(HYBRIDS)
    ladder = [n for n in all_names if not n.startswith("BEST-")]
    pre = [c for c in cells if c["pop"] == "pre-T2464"]
    w2f = [c for c in cells if c["pop"] == "w2f-probes"]
    # positive control: the wrappers reproduce the closed forms to the bit (w5: 1,286/1,286)
    closed = [c for c in pre if models["BEST"][c["k"]][1].startswith("closed")]
    print("control: closed-form cells, BEST bit-exact by function")
    by = defaultdict(lambda: [0, 0])
    for c in closed:
        by[c["fn"]][0] += 1
        by[c["fn"]][1] += models["BEST"][c["k"]][0] == c["excel"]
    for f, (n, x) in sorted(by.items()):
        print(f"  {f:14s} {x}/{n}")
    show("pre-T2464 cells (all functions, cdf + beta pdf)", pre, models, all_names)
    show("pre-T2464 hold-out half (md5 odd)", [c for c in pre if c["hold"]], models, ladder)
    show(
        "pre-T2464, functions other than BETA.DIST (out of sample for the bpser/L140 pieces)",
        [c for c in pre if c["fn"] not in ("BETA.DIST", "BETADIST")],
        models,
        ladder,
    )
    show("pre-T2464, gated suites only", [c for c in pre if c["gated"]], models, ladder)
    show("w2f research probes (tuning data of w2/w3/w4)", w2f, models, ladder)
    for pop, rows in (("pre-T2464", pre), ("w2f-probes", w2f)):
        print(f"\n== {pop}: engine visibly-off cells by BEST branch -> fixed by BEST / BEST+log1p; remaining")
        g = defaultdict(list)
        for c in rows:
            g[branch_of(c, models)].append(c)
        print(
            f"  {'branch':26s}{'n':>7s}{'eng_vis':>9s}{'R_LIKE':>8s}{'TOMS_XL':>8s}{'BEST':>7s}{'B+l1p':>7s}"
            f"{'broke':>7s}{'left':>7s}{'lostex':>8s}"
        )
        for br, rs in sorted(
            g.items(), key=lambda kv: -sum(not lib.within15(c["excel"], c["engine"]) for c in kv[1])
        ):
            vis = [c for c in rs if not lib.within15(c["excel"], c["engine"])]
            fx = {
                m: sum(lib.within15(c["excel"], models[m][c["k"]][0]) for c in vis)
                for m in ("R_LIKE", "TOMS_XL", "BEST", "BEST+log1p")
            }
            broke = sum(
                lib.within15(c["excel"], c["engine"])
                and not lib.within15(c["excel"], models["BEST+log1p"][c["k"]][0])
                for c in rs
            )
            lost = sum(
                c["engine"] == c["excel"] and models["BEST+log1p"][c["k"]][0] != c["excel"] for c in rs
            )
            print(
                f"  {br:26s}{len(rs):7d}{len(vis):9d}{fx['R_LIKE']:8d}{fx['TOMS_XL']:8d}{fx['BEST']:7d}"
                f"{fx['BEST+log1p']:7d}{broke:7d}{len(vis) - fx['BEST+log1p']:7d}{lost:8d}"
            )
    print("\n== pre-T2464: engine visibly-off by function -> fixed by BEST+log1p (left)")
    g = defaultdict(list)
    for c in pre:
        g[c["fn"] + (" pdf" if c["prob"][0] == "pdf" else "")].append(c)
    for f, rs in sorted(g.items()):
        vis = [c for c in rs if not lib.within15(c["excel"], c["engine"])]
        fx = sum(lib.within15(c["excel"], models["BEST+log1p"][c["k"]][0]) for c in vis)
        br = sum(
            lib.within15(c["excel"], c["engine"])
            and not lib.within15(c["excel"], models["BEST+log1p"][c["k"]][0])
            for c in rs
        )
        print(
            f"  {f:18s} n {len(rs):6d} eng_vis {len(vis):5d} fixed {fx:5d} left {len(vis) - fx:5d} broke {br:4d}"
        )


if __name__ == "__main__":
    main()
