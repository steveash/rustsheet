"""Score bgrat_half_ideal on probe t2464-r2-tdist-p1, parsing the formulas from the case file myself.
Part C = rows 3525..8524, part B = rows 725..3524 (checked against the formula shapes). usage: vprobe.py"""

import json
import math
import re
from collections import Counter, defaultdict
from decimal import Decimal

import mpmath

import vlib

ROOT = vlib.T.parents[2] / "eval/suites/probes-numeric/cases"
cells = json.load(open(ROOT / "t2464-r2-tdist-p1.case.json"))["sheets"][0]["cells"]
ora = json.load(open(ROOT / "t2464-r2-tdist-p1.oracle.json"))["sheets"]["Calc"]


def parse_arg(s, mode):
    m = re.fullmatch(r"\((\d+)\*2\^-(\d+)\+(\d+)\*2\^-(\d+)\)", s)
    if m:
        p, k, q, j = map(int, m.groups())
        return p * 2.0**-k + q * 2.0**-j, False
    m = re.fullmatch(r"(\d+)\*2\^-(\d+)", s)
    if m:
        return int(m.group(1)) * 2.0 ** -int(m.group(2)), False
    D = Decimal(s)
    digits = len(D.as_tuple().digits) - (
        len(D.as_tuple().digits) - len(str(D.normalize().as_tuple().digits)) if False else 0
    )
    sig = len(D.normalize().as_tuple().digits)
    if sig > 15 and mode == "r15":
        return float(f"{float(s):.15g}"), True
    if sig > 15 and mode == "t15":
        q = D.normalize()
        e = q.adjusted() - 14
        return float(q.quantize(Decimal(f"1e{e}"), rounding="ROUND_DOWN")), True
    return float(s), sig > 15


def rows(lo, hi, mode):
    out = []
    for i in range(lo, hi + 1):
        f = cells[f"A{i}"]
        m = re.fullmatch(r"=T\.DIST\.RT\((.+),(\d+(?:\.\d+)?)\)", f)
        assert m, f
        t, long_ = parse_arg(m.group(1), mode)
        n = float(m.group(2))
        o = ora.get(f"A{i}")
        if not o or o.get("t") != "n" or o["v"] == 0:
            continue
        y, z = vlib.targs(t, n)
        out.append(dict(i=i, t=t, n=n, y=y, z=z, excel=o["v"], long=long_, br=vlib.branch(n, y, z)))
    return out


def run(sel, **kw):
    vlib.LN([r["y"] for r in sel])
    vlib.ERFC(
        [vlib.needed_s(r["n"], r["y"], r["z"], **{k: v for k, v in kw.items() if k == "cut"}) for r in sel]
    )
    for r in sel:
        r["m"] = vlib.model_I(r["n"], r["y"], r["z"], **kw) * 0.5
    return [(r["m"], r["excel"]) for r in sel]


for mode in ("raw", "r15", "t15"):
    C = rows(3525, 8524, mode)
    print(
        mode,
        "part C rows with value",
        len(C),
        "branches",
        Counter(r["br"] for r in C),
        "long literals",
        sum(r["long"] for r in C),
    )
    sel = [r for r in C if r["br"] == "bgrat"]
    pairs = run(sel)
    h = [p for p, r in zip(pairs, sel) if vlib.myhold(f"A{r['i']}")]
    nl = [p for p, r in zip(pairs, sel) if not r["long"]]
    print(
        f"  bgrat: {vlib.score(pairs)} | my hold half {vlib.score(h)} | exact-literal cells only {vlib.score(nl)}"
    )
    if mode == "raw":
        cr = []
        for r in sel:
            with mpmath.workdps(60):
                T, N = mpmath.mpf(r["t"]), mpmath.mpf(r["n"])
                cr.append(
                    (float(mpmath.betainc(N / 2, 0.5, 0, N / (N + T * T), regularized=True) / 2), r["excel"])
                )
        print("  CR:", vlib.score(cr))
        print("  log1p cells (z<=0.01):", vlib.score([p for p, r in zip(pairs, sel) if r["z"] <= 0.01]))
        print(
            "  misses:",
            [
                (r["i"], r["t"], r["n"], r["long"], f"{(r['m'] - r['excel']) / math.ulp(r['excel']):+.0f}")
                for r in sel
                if r["m"] != r["excel"]
            ],
        )
        # sanity: total in a branch, duplicates
        print("  unique (t,n)", len({(r["t"], r["n"]) for r in sel}))
        for kw in (dict(lnmode="LN"), dict(erfc="cr"), dict(zround=False)):
            if "zround" in kw:
                vlib.ERFC([vlib.needed_s(r["n"], r["y"], r["z"], zround=False) for r in sel])
            if "lnmode" in kw:
                vlib.ERFC([math.sqrt(-(r["n"] / 2 - 0.25) * vlib.C[("LN", r["y"])]) for r in sel])
            pp = [(vlib.model_I(r["n"], r["y"], r["z"], **kw) * 0.5, r["excel"]) for r in sel]
            print("  control", kw, vlib.score(pp))

# part B: which lnx rule fits per cell
B = rows(725, 3524, "raw")
sel = [r for r in B if r["br"] == "bgrat"]
vlib.LN([r["y"] for r in sel])
res = defaultdict(Counter)
for r in sel:
    lnA = vlib.C[("LN", r["y"])]
    lnB = vlib.log1p_xl(-r["z"])
    nu = r["n"] / 2 - 0.25
    vlib.ERFC([math.sqrt(-nu * lnA), math.sqrt(-nu * lnB)])
    vA = (
        float(
            vlib.I_at_ln(r["n"] / 2, lnA)
            * mpmath.mpf(vlib.C[("ERFC", math.sqrt(-nu * lnA))])
            / mpmath.erfc(mpmath.sqrt(-mpmath.mpf(nu) * mpmath.mpf(lnA)))
        )
        / 2
    )
    vB = (
        float(
            vlib.I_at_ln(r["n"] / 2, lnB)
            * mpmath.mpf(vlib.C[("ERFC", math.sqrt(-nu * lnB))])
            / mpmath.erfc(mpmath.sqrt(-mpmath.mpf(nu) * mpmath.mpf(lnB)))
        )
        / 2
    )
    side = "z<=0.01" if r["z"] <= 0.01 else "z>0.01"
    fa, fb = vA == r["excel"], vB == r["excel"]
    res[(r["n"], side)]["both" if fa and fb else "LN" if fa else "log1p" if fb else "none"] += 1
print("part B bgrat cells", len(sel), "of", len(B))
for k in sorted(res):
    print("  ", k, dict(res[k]))
vlib.save()
