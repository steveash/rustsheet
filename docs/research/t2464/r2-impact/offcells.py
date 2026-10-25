"""Step 1: every visibly-off cell of the incomplete-beta family in the eval report, with function, case, cell,
arguments and the engine/Excel values -> offcells.tsv, plus summary tables (stdout -> offcells.out).

Populations: 'standing' = every suite except the untracked round-2 research probes (t2464-r2-*); 'r2probes' = those.
'gated' marks suites whose baseline is gated (eval/suites/*/suite.toml gate = true).
"""

from collections import Counter, defaultdict

import implib as lib


def main() -> None:
    fails, rep = lib.failing_cells()
    args = {(r["case"], r["key"]): r for r in lib.all_cells()}
    fam = [c for c in fails if c["fn"]]
    rows = []
    for c in fam:
        pop = "r2probes" if "/t2464-r2-" in c["case"] else "standing"
        gated = c["suite"] not in lib.UNGATED
        a = args.get((c["case"], c["key"]))
        prob = lib.problem(a["fn"], a["args"]) if a and a["fn"] == c["fn"] else ("none", "unresolved-args")
        rows.append(
            dict(
                c,
                pop=pop,
                gated=gated,
                args=a["args"] if a else None,
                prob=prob,
                single=a is not None and a["fn"] == c["fn"],
            )
        )
    vis = [r for r in rows if lib.is_visible(r["bucket"])]

    print("== report functions table (closeness.visible) for the family, all suites")
    for f in rep["functions"]:
        if f["name"] in lib.FAMILY:
            print(
                f"  {f['name']:18s} cells {f['cells']:6d} visible {f['visible']['numeric'] + f['visible']['other']:6d}"
            )
    print("\n== visibly-off cells by population x function (outer family call)")
    t = Counter((r["pop"], r["fn"]) for r in vis)
    for pop in ("standing", "r2probes"):
        tot = sum(v for (p, _), v in t.items() if p == pop)
        print(f"  [{pop}] total {tot}")
        for (p, f), v in sorted(t.items(), key=lambda kv: -kv[1]):
            if p == pop:
                g = sum(1 for r in vis if r["pop"] == pop and r["fn"] == f and r["gated"])
                print(f"    {f:18s} {v:6d}  (gated suites {g})")
    print("\n== standing visibly-off: by suite")
    for s, v in Counter(r["suite"] for r in vis if r["pop"] == "standing").most_common():
        print(f"  {s:16s} {v}")
    print("\n== standing visibly-off: numeric vs other (error/type)")
    print("  ", Counter(r["bucket"] for r in vis if r["pop"] == "standing"))
    print("\n== standing visibly-off: what the cell is (kernel mapping)")
    kinds = Counter()
    for r in vis:
        if r["pop"] != "standing":
            continue
        p = r["prob"]
        kinds[p[0] if p[0] != "none" else "none:" + p[1]] += 1
    for k, v in kinds.most_common():
        print(f"  {k:28s} {v}")
    print("\n== standing visibly-off by case (top 40)")
    for k, v in Counter(r["case"] for r in vis if r["pop"] == "standing").most_common(40):
        print(f"  {v:6d} {k}")
    with open(lib.HERE / "offcells.tsv", "w") as f:
        f.write("pop\tgated\tsuite\tfn\tcase\tcell\tbucket\tkind\targs\texcel\tengine\tformula\n")
        for r in sorted(vis, key=lambda r: (r["pop"], r["fn"], r["case"], r["key"])):
            p = r["prob"]
            kind = p[0] if p[0] != "none" else "none:" + p[1]
            a = "|".join(repr(x) for x in r["args"]) if r["args"] else ""
            f.write(
                "\t".join(
                    map(
                        str,
                        (
                            r["pop"],
                            int(r["gated"]),
                            r["suite"],
                            r["fn"],
                            r["case"],
                            r["key"],
                            r["bucket"],
                            kind,
                            a,
                            repr(r["excel"]),
                            repr(r["engine"]),
                            r["formula"],
                        ),
                    )
                )
                + "\n"
            )
    print("\nwrote offcells.tsv:", len(vis), "rows")
    # also: all family cells per population, to size the regression risk
    print("\n== standing family cells failing but within 15 digits (ulp1/ulp4/digits15)")
    print(
        "  ", Counter(r["bucket"] for r in rows if r["pop"] == "standing" and not lib.is_visible(r["bucket"]))
    )


if __name__ == "__main__":
    main()
