"""Frozen model LT1 (tdlib.lt1): fit the per-df constant offsets on the in-sample probes a1/a2 (deepest overlap,
all blocks of a df), then score subsets: in-sample probes by branch, old cells.tsv (19,776 cells), and with
--hold the hold-out probe (read once).  Also CR, r2's best forms are in r2-tdist/pdf_lab.out.
usage: s17_final.py [--hold] [--control]"""
import json
import sys
from collections import defaultdict

import mpmath

import tdlib as TD

L = TD.LDB
OFF_F = TD.HERE / "lt1_offsets.json"


def fit(rows, ctl=None):
    by = defaultdict(list)
    for r in rows:
        by[r["n"]].append(r)
    offs = {}
    for n, rr in sorted(by.items()):
        if len(rr) < 50:
            continue
        nb, pos, _, _ = TD.fit_free_C(rr, n, TD.LT1, ctl)
        offs[n] = round(pos, 3)
        print(f"  fit n={n:g}: {nb}/{len(rr)} at {pos:+.3f}")
    return offs


def score(name, rows, pred):
    out = defaultdict(list)
    for r in rows:
        m = pred(r)
        out["all"].append((m, r["excel"]))
        out["br=" + r.get("br", "?")].append((m, r["excel"]))
        out["n=%s" % (f"{r['n']:g}" if r["n"] <= 10 or r["n"] == 30 else ">10")].append((m, r["excel"]))
        if r.get("hold"):
            out["md5-odd"].append((m, r["excel"]))
    for k in sorted(out):
        print(f"{name} {k}: {TD.fmt(TD.tally(out[k]))}")


def branch(r):
    t, n = r["t"], r["n"]
    _, _, b = TD.cache_u(abs(t), n, TD.LT1)
    if b == "l":
        return "log(x2>n)"
    k = n / 2
    xx = TD.st(L(t) * L(t))
    M = TD.st(L(TD.st(L(n) + L(xx))) / 2)
    return "bd0-log" if abs(k - M) >= 0.2 * (k + M) else "bd0-series"


def main():
    ins = [r for r in TD.load_probe("a1") + TD.load_probe("a2")]
    if "--control" in sys.argv:  # positive control: synthetic Excel from LT1 with offsets +3.3 units
        ctl = {id(r): TD.lt1(r["t"], r["n"], 3.3) for r in ins}
        print("control (planted +3.3):")
        fit([r for r in ins if not r["blk"].startswith("df")], ctl)
        return
    print("fitting offsets on a1/a2 (runs and exact blocks):")
    TD.LT1_OFF.update(fit([r for r in ins if not r["blk"].startswith("df")]))
    OFF_F.write_text(json.dumps({f"{k:g}": v for k, v in TD.LT1_OFF.items()}) + "\n")
    for r in ins:
        r["br"] = branch(r)
    cr = lambda r: float(mpmath.exp(TD.exact_logpdf(r["t"], r["n"])))  # noqa: E731
    score("in-sample LT1", ins, lambda r: TD.lt1(r["t"], r["n"]))
    score("in-sample CR", ins, cr)
    old = [r for r in TD.load_old() if r["n"] >= 3]
    for r in old:
        r["br"] = branch(r)
    score("old LT1", old, lambda r: TD.lt1(r["t"], r["n"]))
    score("old LT1 exactC", old, lambda r: TD.lt1(r["t"], r["n"], 0.0))
    score("old CR", old, lambda r: TD.exact_pdf(r["t"], r["n"]))
    if "--hold" in sys.argv:
        hold = TD.load_probe("hold")
        for r in hold:
            r["br"] = branch(r)
        score("hold LT1", hold, lambda r: TD.lt1(r["t"], r["n"]))
        score("hold CR", hold, cr)
        print("hold-out structure test (offset fitted on the hold-out's md5-even half, scored on the odd half):")
        ev = [r for r in hold if not r["hold"] and not r["blk"].startswith("df")]
        offs = fit(ev)
        od = [r for r in hold if r["hold"] and r["n"] in offs]
        score("hold-odd LT1(free C)", od, lambda r: TD.lt1(r["t"], r["n"], offs[r["n"]]))
        score("hold-odd CR", od, cr)
    TD.save_exact()


if __name__ == "__main__":
    main()
