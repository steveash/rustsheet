"""Final scores of excel_log1p (this worker's model) against: (1) pinned l per case and p region; (2) the probe
cells themselves, BINOM.DIST(0,2^k,p) = EXP(2^k*l); (3) out-of-sample: every BINOM.DIST(0,n,p) cell in
../cells.tsv (general n, = EXP(fl(n*l)) per w5), which no choice here was tuned on.
Counts are bit-exact / within 4 ulp / within 15 significant digits.  Usage: final_score.py"""
import csv
import math

import cands
import excel_log1p as XL
import pin
import r2lib as R

REG = [(0, 1e-4), (1e-4, 0.05), (0.05, 0.15), (0.15, 0.25), (0.25, 0.375 + 1e-17), (0.375 + 1e-17, 0.6)]


def w15(m, e):
    if m == e:
        return True
    if not (math.isfinite(m) and math.isfinite(e)) or e == 0:
        return False
    return abs(m - e) <= 0.5 * 10.0 ** (math.floor(math.log10(abs(e))) - 14)


def tri(pairs):
    pairs = list(pairs)
    ex = sum(m == e for m, e in pairs)
    u4 = sum(m == e or (math.isfinite(m) and abs(m - e) <= 4 * math.ulp(e)) for m, e in pairs)
    return f"{ex}/{u4}/{sum(w15(m, e) for m, e in pairs)} of {len(pairs)}"


def model_l(p):
    return XL.ln1m(p)


def main():
    print("(1) pinned l = Excel's ln(1-p): model vs CR log1p, per case and p region")
    allp = []
    for nm in ("runs", "grid", "hold", "tiny", "pilot"):
        rows = [x for x in pin.load(nm) if x[2] not in ("none", "multi")]
        allp += [(nm, x) for x in rows if nm != "pilot"]
        for lo, hi in REG:
            sel = [x for x in rows if lo <= x[0] < hi]
            if sel:
                print(f"  {nm:5s} p in [{lo:.3g},{hi:.3g})  model {tri((model_l(p), l) for p, n, k, l in sel):24s}"
                      f" CR-log1p {tri((R.cr_log1p(-p) if p <= 0.375 else cands.ln_fl(-p), l) for p, n, k, l in sel)}")
    for h in (0, 1):
        sel = [x for i, (nm, x) in enumerate(allp) if i % 2 == h]
        print(f"  all cases, {'even' if h == 0 else 'odd'} rows: model {tri((model_l(p), l) for p, n, k, l in sel)}")
    print("(2) probe cells BINOM.DIST(0,2^k,p,FALSE) (column B), predicted EXP(2^k * model l)")
    for nm in ("runs", "grid", "hold"):
        o = R.read_oracle(nm)
        import make_probes as MP
        ps = MP.sets(nm)
        pr = [(p, R.binom_n(p), o.get(f"B{i + 1}")) for i, p in enumerate(ps)]
        pr = [x for x in pr if isinstance(x[2], float)]
        ex = R.xl("EXP", [n * model_l(p) for p, n, v in pr])
        print(f"  {nm:5s} {tri(zip(ex, [v for _, _, v in pr]))}")
    print("(3) out of sample: BINOM.DIST(0,n,p,*) cells in ../cells.tsv, = EXP(fl(n*l))")
    rows = []
    for r in csv.DictReader(open(R.HERE.parent / "cells.tsv"), delimiter="\t"):
        if r["fn"] not in ("BINOM.DIST", "BINOMDIST"):
            continue
        a = r["args"].split("|")
        try:
            k, n, p, ex = math.trunc(float(a[0])), math.trunc(float(a[1])), float(a[2]), float(r["excel"])
        except (ValueError, IndexError):
            continue
        if k == 0 and 0 < p < 1 and n >= 1:
            rows.append((p, n, ex))
    for nm, f in (("model", model_l), ("CR log1p / LN(1-p)", lambda p: R.cr_log1p(-p) if p <= 0.375 else cands.ln_fl(-p))):
        pred = R.xl("EXP", [R.excelmath.x87("*", float(n), f(p)) for p, n, ex in rows])
        lo = [i for i, x in enumerate(rows) if x[0] <= 0.375]
        print(f"  {nm:20s} all {tri(zip(pred, [x[2] for x in rows]))}   p<=0.375 {tri((pred[i], rows[i][2]) for i in lo)}")


if __name__ == "__main__":
    main()
