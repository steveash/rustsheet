"""Analyse probe t2464-r2-tdist-p1 (see make_probe.py). usage: p1.py [A|B|C|D ...]"""

import json
import sys
from collections import defaultdict

import mpmath

import tails
import tlib
import tscore
import xlk

bx = tlib.bx
NAME = "t2464-r2-tdist-p1"
META = json.load(open(tlib.HERE / f"{NAME}.meta.json"))


def _seen(v: float) -> float:
    """The double Excel evaluated: w2-forensics' num() writes repr(v) when it thinks it has <= 15 digits, but its
    digit count drops a '0.' inside numbers >= 10 (10.88270147083634 has 16), and the formula parser cuts number
    literals to 15 significant digits."""
    r = repr(v)
    if len(r.replace("0.", "").replace(".", "")) <= 15 and float(r) == v:
        return float(f"{v:.15g}")
    return v


META = [(i, p, [_seen(v) for v in args]) for i, p, args in META]
ORA = json.load(open(tlib.T2464.parents[2] / "eval/suites/probes-numeric/cases" / f"{NAME}.oracle.json"))[
    "sheets"
]["Calc"]
FINAL = "args=t2,alnrel=xl,erfc=xl,bg_lnx=cut0.01,bg=ideal"


def excel(i):
    c = ORA.get(f"A{i}")
    return c["v"] if c and c.get("t") == "n" else None


def part_A():
    """bup + bgrat with Excel's own density (column A) as bup's prefactor: I = RN(P y z / a W + G)."""
    P = {(y, n): excel(i) for i, part, (y, n) in META if part == "A"}
    rows = tscore.rows_L()
    vals, brs = tscore.evaluate(rows, FINAL)
    cfg = tscore.parse(FINAL)[1]
    K = bx._KERNELS[cfg]
    out = defaultdict(list)
    dens = []
    for r, v, b in zip(rows, vals, brs):
        if b != "bup+bgrat/sw":
            continue
        n = r["n"]
        a = n / 2
        sc = 1 if r["kind"] == "2t" else 0.5
        tt, u, y, z = tlib.targs(abs(r["t"]), n)
        p = P[(y, n)]
        pm, _ = K.density(y, a, 0.5)
        dens.append((pm, p))
        with mpmath.workdps(50):
            G = xlk.bgrat_half_ideal(a + 20, bx.LN(y))
            Y, Z = mpmath.mpf(y), mpmath.mpf(z)
            W = sum(mpmath.rf(a + 0.5, k) / mpmath.rf(a + 1, k) * Y**k for k in range(20))
            for name, pp in (("excelP", p), ("M1P", pm)):
                B = mpmath.mpf(pp) * Y * Z / a * W
                out[name].append((float((B + G) * sc), r["excel"]))
                out[(name, "hold" if r["hold"] else "tune")].append((float((B + G) * sc), r["excel"]))
                out[(name, "Pyz stored")].append(
                    (float((mpmath.mpf(bx.sx("*", pp, bx.sx("*", y, z))) / a * W + G) * sc), r["excel"])
                )
        out["FINAL model"].append((v, r["excel"]))
    print(tlib.line("w1 M1 density vs Excel BETA.DIST(y,n/2,0.5,FALSE)", dens))
    for k in sorted(out, key=str):
        print(tlib.line(str(k), out[k]))


def part_B():
    """bgrat's lnx switch at z = 0.01: per n, which rule fits each cell (cut at 0.01 / always LN(y) / always
    log1p(-z)); prints the z ranges where each wins. A first pass fills Excel's ERFC cache (lazy lookups)."""
    for i, p, (t, nn) in META:
        if p == "B":
            tt, u, y, z = tlib.targs(t, nn)
            for m in (bx.LN(y), xlk.excel_log1p.log1p_series(-z)):
                xlk.bgrat_half_ideal(nn / 2, m)
    xlk.englib.fetch_missing()
    for nn in sorted({n for i, p, (t, n) in META if p == "B"}):
        cells = [(i, t) for i, p, (t, n) in META if p == "B" and n == nn]
        res = []
        for i, t in cells:
            e = excel(i)
            tt, u, y, z = tlib.targs(t, nn)
            v = {}
            for lab, m in (("LN", bx.LN(y)), ("l1p", xlk.excel_log1p.log1p_series(-z))):
                with mpmath.workdps(50):
                    v[lab] = float(xlk.bgrat_half_ideal(nn / 2, m) / 2)
            res.append((z, y, v["LN"] == e, v["l1p"] == e))
        res.sort()
        both = sum(a and b for _, _, a, b in res)
        lnonly = [z for z, y, a, b in res if a and not b]
        l1only = [z for z, y, a, b in res if b and not a]
        none = sum(not a and not b for _, _, a, b in res)
        print(
            f"n={nn:g} cells {len(res)} both {both} none {none} | LN-only z in "
            f"[{min(lnonly, default=float('nan')):.17g}, {max(lnonly, default=float('nan')):.17g}] ({len(lnonly)}) | "
            f"log1p-only z in [{min(l1only, default=float('nan')):.17g}, {max(l1only, default=float('nan')):.17g}] ({len(l1only)})"
        )


def part_CD(part):
    rows = []
    for i, p, (t, n) in META:
        if p == part and excel(i) not in (None, 0.0):
            rows.append(dict(kind="rt", t=t, n=n, excel=excel(i), hold=tlib.hold(NAME, f"A{i}")))
    for r in rows:
        r["cr"] = tlib.exact_cell(r)
    tlib.save_exact()
    specs = {"FINAL": FINAL, "r2-bratio BEST, yz": "args=yz", "FINAL bg=ld": FINAL.replace("ideal", "ld")}
    out = defaultdict(list)
    for name, s in specs.items():
        vals, brs = tscore.evaluate(rows, s)
        if name == "FINAL":
            br0 = brs
        for r, v, b in zip(rows, vals, br0 if name != "FINAL" else brs):
            out[(name, b)].append((v, r["excel"]))
            out[(name, "ALL")].append((v, r["excel"]))
            if r["hold"]:
                out[(name, "ALL hold")].append((v, r["excel"]))
    for r, b in zip(rows, br0):
        out[("CR", b)].append((r["cr"], r["excel"]))
        out[("CR", "ALL")].append((r["cr"], r["excel"]))
    xlk.save_cache()
    for k in sorted(out, key=lambda k: (k[1], k[0])):
        print(tlib.line(str(k), out[k]))


if __name__ == "__main__":
    for p in sys.argv[1:] or ["A", "B", "C", "D"]:
        print(f"== part {p}")
        part_A() if p == "A" else part_B() if p == "B" else part_CD(p)
