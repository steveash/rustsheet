"""Independent verifier for r2-log1p (T-2464). Own x87 rounding (Fractions), own model from the description,
own parsing of the case formulas and oracle values, scored directly against BINOM.DIST cells (no pins reused).
Usage: python vscore.py"""
import csv, json, math, re, sys, hashlib
from fractions import Fraction as F
from pathlib import Path
HERE = Path(__file__).parent
sys.path.insert(0, str(HERE.parent))
import excelmath  # Excel EXP/LN models (shared harness, unavoidable)
import mpmath
mpmath.mp.prec = 200
SUITE = HERE.parents[3] / "eval/suites/probes-numeric/cases"

def rnd(q: F, bits: int) -> F:
    if q == 0: return q
    s = -1 if q < 0 else 1; q = abs(q)
    e = q.numerator.bit_length() - q.denominator.bit_length()
    if F(2) ** e > q: e -= 1
    # q in [2^e, 2^(e+1)); scale so integer part has `bits` bits
    sc = bits - 1 - e
    v = q * (F(2) ** sc)
    n, r = divmod(v.numerator, v.denominator)
    rem = F(r, v.denominator)
    if rem > F(1, 2) or (rem == F(1, 2) and n % 2): n += 1
    return s * F(n) / (F(2) ** sc)

def X(op, a, b):
    a, b = F(a), F(b)
    v = {"+": a + b, "-": a - b, "*": a * b, "/": a / b}[op]
    return float(rnd(rnd(v, 64), 53))

C = [1.0 / (2 * k + 1) for k in range(16)]

def model(p, N=14, order="fwd", horner=False, quot="x87"):
    a = -p
    s2 = a + 2.0
    r = X("/", a, s2) if quot == "x87" else a / s2
    t2 = X("*", r, r)
    if horner:  # S = c1 + t2*(c2 + ...); s = r*t2*S
        S = C[N]
        for k in range(N - 1, 0, -1):
            S = X("+", C[k], X("*", t2, S))
        s = X("*", X("*", r, t2), S)
    else:
        terms, pk = [], r
        for k in range(1, N + 1):
            pk = X("*", pk, t2); terms.append(X("*", pk, C[k]))
        if order == "rev": terms = terms[::-1]
        s = terms[0]
        for t in terms[1:]: s = X("+", s, t)
    return 2 * X("+", r, s)

def ln1m(p, **kw):
    return model(p, **kw) if p <= 0.375 else None  # >0.375 handled with Excel LN

LIT = re.compile(r"^=\((\d+)\*2\^(-?\d+)\+(\d+)\*2\^(-?\d+)\)$")
def parse_lit(f):
    m = LIT.match(f)
    if not m: return None
    h, e1, l, e2 = map(int, m.groups())
    v = F(h) * F(2) ** e1 + F(l) * F(2) ** e2
    assert float(v) == v
    return float(v)

def load_case(name):
    c = json.loads((SUITE / f"t2464-r2-log1p-{name}.case.json").read_text())["sheets"][0]["cells"]
    o = json.loads((SUITE / f"t2464-r2-log1p-{name}.oracle.json").read_text())["sheets"]["Calc"]
    rows = []
    i = 1
    while f"A{i}" in c:
        p = parse_lit(c[f"A{i}"])
        mb = re.match(r"^=BINOM\.DIST\(0,(\d+),A%d,FALSE\)$" % i, c[f"B{i}"])
        assert p is not None and mb, (name, i)
        n = int(mb.group(1))
        ea, eb = o.get(f"A{i}", {}), o.get(f"B{i}", {})
        echo = ea.get("v") == p
        bv = eb.get("v") if eb.get("t") == "n" else ("ERR:" + str(eb.get("v")))
        rows.append(dict(case=name, i=i, p=p, n=n, B=bv, echo=echo))
        i += 1
    return rows

def main():
    allrows = []
    for nm in ("pilot", "runs", "grid", "hold"):
        rs = load_case(nm)
        print(nm, "rows", len(rs), "echo mismatch", sum(not r["echo"] for r in rs),
              "errors", sum(isinstance(r["B"], str) for r in rs),
              "n of errors", sorted({r["n"] for r in rs if isinstance(r["B"], str)})[:3],
              "max finite n", max(r["n"] for r in rs if not isinstance(r["B"], str)))
        allrows += rs
    rows = [r for r in allrows if not isinstance(r["B"], str) and r["echo"] and 0 < r["p"] < 1]
    # duplicates (same p across/within cases)
    seen = {}
    for r in rows: seen.setdefault(r["p"], []).append(r["case"])
    dup = {p: v for p, v in seen.items() if len(v) > 1}
    print("finite rows", len(rows), "distinct p", len(seen), "dup p", len(dup),
          "dup by case pair", sorted({tuple(v) for v in dup.values()})[:6])
    # n*l must be exact (power of two n)
    assert all(r["n"] & (r["n"] - 1) == 0 for r in rows)
    variants = {"model": {}, "horner": dict(horner=True), "rev": dict(order="rev"),
                "ieee_quot": dict(quot="ieee"), "N11": dict(N=11), "N12": dict(N=12)}
    lnv = excelmath.excel("LN", [1.0 - r["p"] for r in rows])
    preds = {}
    for vn, kw in variants.items():
        ls = [model(r["p"], **kw) if r["p"] <= 0.375 else lnv[j] for j, r in enumerate(rows)]
        preds[vn] = ls
    preds["CR"] = [float(mpmath.log1p(-mpmath.mpf(r["p"]))) if r["p"] <= 0.375 else lnv[j] for j, r in enumerate(rows)]
    # also cross-check against the worker's own function
    sys.path.insert(0, str(HERE.parent / "r2-log1p"))
    import excel_log1p as W
    wl = [W.ln1m(r["p"]) for r in rows]
    print("worker excel_log1p.ln1m disagrees with my model on", sum(a != b for a, b in zip(wl, preds["model"])), "rows")
    ex = {}
    for vn, ls in preds.items():
        ex[vn] = excelmath.excel("EXP", [r["n"] * l for r, l in zip(rows, ls)])
    def hsplit(r):  # hash split independent of row order
        return hashlib.sha1(repr(r["p"]).encode()).digest()[0] & 1
    subsets = {
        "all (incl pilot)": lambda r: True,
        "all excl pilot": lambda r: r["case"] != "pilot",
        "p<=0.375 excl pilot": lambda r: r["case"] != "pilot" and r["p"] <= 0.375,
        "p>0.375": lambda r: r["p"] > 0.375,
        "runs+grid only, p<=0.375": lambda r: r["case"] in ("runs", "grid") and r["p"] <= 0.375,
        "hold only, p<=0.375": lambda r: r["case"] == "hold" and r["p"] <= 0.375,
        "pilot only": lambda r: r["case"] == "pilot",
        "hash half 0 p<=.375": lambda r: r["case"] != "pilot" and r["p"] <= 0.375 and hsplit(r) == 0,
        "hash half 1 p<=.375": lambda r: r["case"] != "pilot" and r["p"] <= 0.375 and hsplit(r) == 1,
        "p in [0.25,0.375]": lambda r: r["case"] != "pilot" and 0.25 <= r["p"] <= 0.375,
    }
    def w15(m, e):
        return m == e or (e != 0 and abs(m - e) <= 0.5 * 10.0 ** (math.floor(math.log10(abs(e))) - 14))
    for sn, f in subsets.items():
        idx = [j for j, r in enumerate(rows) if f(r)]
        line = f"{sn:28s} n={len(idx):5d}"
        for vn in ex:
            e = sum(ex[vn][j] == rows[j]["B"] for j in idx)
            line += f" {vn}={e}"
        print(line)
    # within 4 ulp of l: invert per row to admissible l set near the model's l
    idx = [j for j, r in enumerate(rows) if r["case"] != "pilot"]
    miss = [j for j in idx if ex["model"][j] != rows[j]["B"]]
    cand = []
    for j in miss:
        l = preds["model"][j]
        for k in range(-6, 7):
            x = l
            for _ in range(abs(k)): x = math.nextafter(x, math.copysign(math.inf, k))
            cand.append((j, k, x))
    ce = excelmath.excel("EXP", [rows[j]["n"] * x for j, k, x in cand])
    offs = {}
    for (j, k, x), v in zip(cand, ce):
        if v == rows[j]["B"]: offs.setdefault(j, []).append(k)
    hist = {}
    for j in miss:
        key = tuple(offs.get(j, [])); hist[key] = hist.get(key, 0) + 1
    print("model misses (excl pilot):", len(miss), "offset histogram of admissible l (ulps of l):", hist)
    print("miss cases:", {c: sum(rows[j]['case'] == c for j in miss) for c in ('runs', 'grid', 'hold')})
    # out-of-sample cells.tsv BINOM k=0
    print("--- cells.tsv BINOM k=0 ---")
    oos = []
    for r in csv.DictReader(open(HERE.parent / "cells.tsv"), delimiter="\t"):
        if r["fn"] not in ("BINOM.DIST", "BINOMDIST"): continue
        a = r["args"].split("|")
        try:
            k, n, p, e = float(a[0]), float(a[1]), float(a[2]), float(r["excel"])
        except Exception: continue
        if math.trunc(k) == 0 and 0 < p < 1 and math.trunc(n) >= 1:
            oos.append((math.trunc(n), p, e, r["case"], r["cell"], a[3] if len(a) > 3 else None))
    keys = {(n, p) for n, p, *_ in oos}
    print("rows", len(oos), "distinct (n,p)", len(keys), "distinct p", len({p for _, p, *_ in oos}),
          "p<=0.375", sum(p <= 0.375 for _, p, *_ in oos), "distinct (n,p) p<=.375", len({(n, p) for n, p, *_ in oos if p <= 0.375}))
    lnv2 = excelmath.excel("LN", [1 - p for _, p, *_ in oos])
    res = {}
    for vn, kw in (("model", {}), ("horner", dict(horner=True)), ("CR", None)):
        ls = []
        for (n, p, *_), lnp in zip(oos, lnv2):
            if p > 0.375: ls.append(lnp)
            elif kw is None: ls.append(float(mpmath.log1p(-mpmath.mpf(p))))
            else: ls.append(model(p, **kw))
        res[vn] = (ls, excelmath.excel("EXP", [X("*", float(n), l) for (n, *_), l in zip(oos, ls)]))
    lo = [j for j, x in enumerate(oos) if x[1] <= 0.375]
    for vn, (ls, pr) in res.items():
        print(f"  {vn}: all {sum(pr[j] == oos[j][2] for j in range(len(oos)))}/{len(oos)}  p<=.375 {sum(pr[j] == oos[j][2] for j in lo)}/{len(lo)}")
    disc = [j for j in lo if res["model"][0][j] != res["CR"][0][j]]
    disc2 = [j for j in disc if res["model"][1][j] != res["CR"][1][j]]
    print("  p<=.375 rows where model l != CR l:", len(disc), "; where predicted cell differs:", len(disc2),
          "; model right on those:", sum(res["model"][1][j] == oos[j][2] for j in disc2))
    dh = [j for j in lo if res["model"][1][j] != res["horner"][1][j]]
    print("  p<=.375 rows where model vs horner prediction differs:", len(dh), "model right:", sum(res["model"][1][j] == oos[j][2] for j in dh))
    # do any cells.tsv BINOM p values coincide with probe p's?
    probe_p = {r["p"] for r in allrows}
    print("  cells.tsv k=0 p that also appear in probes:", sum(p in probe_p for _, p, *_ in oos))
    print("  distinct n values (p<=.375):", sorted({oos[j][0] for j in lo})[:30])

if __name__ == "__main__":
    main()
