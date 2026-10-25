"""w6-literature H2d: R <= 2.7.1 dbinom_raw (lf = log(M_2PI) + log(x) + log(n-x) - log(n), log(M_2PI) a runtime
call) inside R dbeta's a,b > 2 branch, stored x87 ops, Excel LN/EXP. Variants: ln(2pi) constant vs runtime LN;
lc accumulated stored (D) or extended (E) with the rest stored. Exact counts per block, no tuning."""
import collections, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from h2_loader import M_LN_2PI, M_2PI, Arith, D, R64, exp_v6, load_rows, mpf  # noqa: E402

def E_lf0(A, x, a, b, ln2pi_runtime, lc_ext):
    r = A.r
    x, a, b = mpf(x), mpf(a), mpf(b)
    k = r(a - 1); n = r(r(a + b) - 2); y = mpf(D(1 - x)); nk = r(n - k)
    rr = R64 if lc_ext else r
    lc = rr(rr(rr(rr(A.stirlerr(n) - A.stirlerr(k)) - A.stirlerr(nk)) - A.bd0(k, r(n * x))) - A.bd0(nk, r(n * y)))
    l2p = A.ln(M_2PI) if ln2pi_runtime else mpf(M_LN_2PI)
    lf = r(r(r(l2p + A.ln(k)) + A.ln(nk)) - A.ln(n))
    return r(A.ln(r(r(a + b) - 1)) + r(lc - r(0.5 * lf)))

rows = load_rows()
by = collections.defaultdict(list)
for r in rows:
    by[r["blk"]].append(r)
A = Arith("D", "xl", "cr", "lnG")
print("Excel LN(2pi) - M_LN_2PI =", float(A.ln(M_2PI) - mpf(M_LN_2PI)))
for k in sorted(by):
    rs = by[k][:300]
    out = [f"{k:18s} a={rs[0]['a']:.6g} b={rs[0]['b']:.6g} n={len(rs)} CR {sum(r['exact']==r['excel'] for r in rs)}"]
    for rt in (False, True):
        for le in (False, True):
            m = [exp_v6(E_lf0(A, r["x"], r["a"], r["b"], rt, le)) for r in rs]
            ev = sum(m[i] == rs[i]["excel"] for i in range(0, len(rs), 2)); od = sum(m[i] == rs[i]["excel"] for i in range(1, len(rs), 2))
            out.append(f"   ln2pi={'LN' if rt else 'const'} lc={'ext' if le else 'stored'}: {ev+od} (e{ev} o{od})")
    print("\n".join(out)); sys.stdout.flush()
