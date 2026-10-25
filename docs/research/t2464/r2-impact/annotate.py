"""Steps 3-4: annotate every pre-T2464 visibly-off cell with the bratio branch, the engine piece that would fix it
and whether the round-1/2 model of that piece does (within 15 digits) -> offcells_pre.tsv; per-piece totals; and the
PLAN end state (BEST + r2-log1p, r2-density-large's LDe for the min > 1 density, and the current engine kept for
T.DIST* in the bgrat family, where BEST is worse than the engine). Reads impact.pkl and offcells.tsv.
"""

import csv
from collections import Counter, defaultdict

import impact
import implib as lib
import pdf_large

_PIECES = {
    "P1 closed forms": "closed:b0=1|closed:a0=1",
    "P2 density M1 + Excel log1p": "pdf:M1 min<=1|pdf:M1 b=1",
    "P4 bpser (w3 series)": "w_bpser|w1_bpser|bpser(a,b>1)",
    "P5 L140 bup, brcmp1 = pdf*x*y": "L140 bup+bpser",
    "P6 density Loader (min > 1)": "pdf:min>1,max<8|pdf:min>1,max>=8",
    "P7 bfrac with brcomp = pdf*x*y": "bfrac|basym",
    "U1 bgrat family (unsolved)": "bgrat|bup+bgrat|L140 bup+bgrat|L140 bup+bup+bgrat",
}
PIECE = {branch: piece for piece, branches in _PIECES.items() for branch in branches.split("|")}
BGRAT = {"bgrat", "bup+bgrat", "L140 bup+bgrat", "L140 bup+bup+bgrat"}


def plan_value(c, models, lde):
    br = impact.branch_of(c, models)
    if lib.FAMILY.get(c["fn"]) == "t" and br in BGRAT:
        return c["engine"]
    if br.startswith("pdf:min>1"):
        return lde[c["k"]]
    return models["BEST+log1p"][c["k"]][0]


def main() -> None:
    cells, models = lib.cache("impact", impact.build)
    lde = {}
    for c in cells:
        if c["prob"][0] == "pdf" and min(c["prob"][2], c["prob"][3]) > 1:
            x, a, b, width = c["prob"][1:]
            v = float(pdf_large.LDM.density(x, a, b, pdf_large.OPTS))
            lde[c["k"]] = v / width if width != 1.0 else v
    byk = {c["k"]: c for c in cells}
    out = []
    per = defaultdict(Counter)
    with open(lib.HERE / "offcells.tsv") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            if r["pop"] != "standing" or "/t2464-" in r["case"]:
                continue
            c = byk.get((r["case"], r["cell"]))
            if r["bucket"] == "visible_other":
                piece, branch, mv = "X argument/error handling (not the kernel)", "", None
            elif c is not None:
                branch = impact.branch_of(c, models)
                piece = PIECE.get(branch, branch)
                mv = plan_value(c, models, lde)
            elif r["kind"] == "none:inverse":
                piece, branch, mv = "P8 inverse: crossing search on the ported CDF", "", None
            elif r["kind"] in (
                "none:f-pdf",
                "none:t-pdf",
                "none:binom-pmf",
                "none:negbinom-pmf",
                "none:binom-k0",
            ):
                piece, branch, mv = "O other kernel (" + r["kind"][5:] + ")", "", None
            else:
                piece, branch, mv = "K same kernel, arguments not resolved here (nested/ref)", "", None
            fixed = "" if mv is None else int(lib.within15(float(r["excel"]), mv))
            per[piece]["n"] += 1
            per[piece]["gated"] += r["gated"] == "1"
            if fixed != "":
                per[piece]["modelled"] += 1
                per[piece]["fixed"] += fixed
            out.append(
                [
                    r["fn"],
                    r["case"],
                    r["cell"],
                    r["gated"],
                    r["bucket"],
                    branch,
                    piece,
                    r["args"],
                    r["excel"],
                    r["engine"],
                    "" if mv is None else repr(mv),
                    str(fixed),
                    r["formula"],
                ]
            )
    with open(lib.HERE / "offcells_pre.tsv", "w") as f:
        f.write(
            "fn\tcase\tcell\tgated\tbucket\tbranch\tpiece\targs\texcel\tengine\tplan_model\tplan_fixes\tformula\n"
        )
        for row in sorted(out, key=lambda r: (r[6], r[0], r[1], r[2])):
            f.write("\t".join(row) + "\n")
    print(f"pre-T2464 visibly-off cells: {len(out)} (offcells_pre.tsv)")
    print(f"  {'piece':58s}{'n':>6s}{'gated':>7s}{'modelled':>9s}{'fixed':>7s}")
    for p, t in sorted(per.items()):
        print(f"  {p:58s}{t['n']:6d}{t['gated']:7d}{t['modelled']:9d}{t['fixed']:7d}")
    for pop in ("pre-T2464", "w2f-probes"):
        rs = [c for c in cells if c["pop"] == pop]
        for label, sub in (
            ("all", rs),
            ("gated", [c for c in rs if c["gated"]]),
            ("hold-out", [c for c in rs if c["hold"]]),
        ):
            if not sub:
                continue
            t = impact.tallies(sub, lambda c: plan_value(c, models, lde))
            print(f"PLAN {pop:10s} {label:8s} " + " ".join(f"{k}={t[k]}" for k in impact.COLS))


if __name__ == "__main__":
    main()
