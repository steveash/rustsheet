"""Step 4: the engine port order as a cumulative ladder. Each step switches one more group of bratio branches from
the current engine to the round-1/2 model (BEST + r2-log1p; LDe for the min > 1 density), the rest stays on the
engine. Prints, per step, the cumulative and incremental visibly-off change, bit-exact cells lost (all / gated), on
pre-T2464 cells, their md5 hold-out half, the non-BETA functions, and the w2f probes. Reads impact.pkl.
"""

import annotate
import impact
import implib as lib
import pdf_large

STEPS = [
    ("S1 closed forms (P1)", lambda br, fam: br.startswith("closed")),
    ("S2 density M1 + log1p (P2)", lambda br, fam: br.startswith("pdf:M1")),
    ("S3 bratio dispatch + bpser (P3+P4)", lambda br, fam: br in ("w_bpser", "w1_bpser", "bpser(a,b>1)")),
    ("S4 L140 bup+bpser (P5)", lambda br, fam: br == "L140 bup+bpser"),
    ("S5 bfrac/basym (P7)", lambda br, fam: br in ("bfrac", "basym")),
    ("S6 bgrat family, not T (U1 as TOMS)", lambda br, fam: br in annotate.BGRAT and fam != "t"),
    ("S7 density Loader min>1 (P6)", lambda br, fam: br.startswith("pdf:min>1")),
    ("(S6' bgrat family for T too)", lambda br, fam: br in annotate.BGRAT and fam == "t"),
]


def main() -> None:
    cells, models = lib.cache("impact", impact.build)
    lde = {}
    for c in cells:
        if c["prob"][0] == "pdf" and min(c["prob"][2], c["prob"][3]) > 1:
            x, a, b, width = c["prob"][1:]
            v = float(pdf_large.LDM.density(x, a, b, pdf_large.OPTS))
            lde[c["k"]] = v / width if width != 1.0 else v
    br = {c["k"]: impact.branch_of(c, models) for c in cells}

    def model_value(c):
        return lde[c["k"]] if br[c["k"]].startswith("pdf:min>1") else models["BEST+log1p"][c["k"]][0]

    subsets = [
        ("pre-T2464", [c for c in cells if c["pop"] == "pre-T2464"]),
        ("pre hold-out", [c for c in cells if c["pop"] == "pre-T2464" and c["hold"]]),
        ("pre non-BETA", [c for c in cells if c["pop"] == "pre-T2464" and lib.FAMILY.get(c["fn"]) != "beta"]),
        ("pre gated", [c for c in cells if c["pop"] == "pre-T2464" and c["gated"]]),
        ("w2f probes", [c for c in cells if c["pop"] == "w2f-probes"]),
    ]
    for name, rows in subsets:
        print(
            f"\n== {name}: n {len(rows)}, engine visibly off {sum(not lib.within15(c['excel'], c['engine']) for c in rows)}"
        )
        print(
            f"  {'step':40s}{'vis_left':>9s}{'d_vis':>7s}{'fixed':>7s}{'broke':>7s}{'exact':>7s}{'lost_ex':>8s}"
            f"{'lost_gated':>11s}"
        )
        on = []
        prev = None
        for label, pred in STEPS:
            on = on + [pred] if not label.startswith("(") else on + [pred]
            sel = list(on) if not label.startswith("(") else list(on)

            def val(c, sel=sel):
                fam = lib.FAMILY.get(c["fn"])
                return model_value(c) if any(p(br[c["k"]], fam) for p in sel) else c["engine"]

            t = impact.tallies(rows, val)
            left = t["eng_vis"] - t["fixed"] + t["broke15"]
            print(
                f"  {label:40s}{left:9d}{(left - prev) if prev is not None else left - t['eng_vis']:7d}"
                f"{t['fixed']:7d}{t['broke15']:7d}{t['ex']:7d}{t['lost_exact']:8d}{t['lost_exact_gated']:11d}"
            )
            prev = left


if __name__ == "__main__":
    main()
