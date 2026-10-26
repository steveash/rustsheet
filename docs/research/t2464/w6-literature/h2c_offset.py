"""w6-literature H2c: Loader density (stored x87 ops, Excel LN/EXP) plus ONE free constant per (a, b) block.

If Excel's a, b > 2 density were Loader's dbinom_raw with different per-shape constants (stirlerr values,
lnB-like terms), a single additive constant c on the exponent E would make each block bit-exact.
For each block: scan c (units of 2^-53, absolute in E) and report the best exact count, with c tuned on the
even rows and scored on the odd rows (hold-out). Control: the exact log-density (extended) with its own best c
(a smooth model with a free constant: the plateau any near-miss reaches).
"""

import collections
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import mpmath  # noqa: E402

from h2_loader import M_LN_2PI, Arith, D, R64, exp_v6, load_rows, mpf  # noqa: E402

U = mpf(2) ** -53


def loader_E(A, x, a, b, lf_form):
    r = A.r
    x, a, b = mpf(x), mpf(a), mpf(b)
    k = r(a - 1)
    n = r(r(a + b) - 2)
    y = mpf(D(1 - x))
    nk = r(n - k)
    lc = r(r(r(r(A.stirlerr(n) - A.stirlerr(k)) - A.stirlerr(nk)) - A.bd0(k, r(n * x))) - A.bd0(nk, r(n * y)))
    f = r(r(a + b) - 1)
    if lf_form == "LF1":
        lf = r(r(M_LN_2PI + A.ln(k)) + A.log1p(r(-k / n)))
    else:
        lf = r(r(r(M_LN_2PI + A.ln(k)) + A.ln(nk)) - A.ln(n))
    return r(A.ln(f) + r(lc - r(0.5 * lf)))


def best_c(Es, ex, idx, cs):
    best = (-1, 0)
    for c in cs:
        k = sum(exp_v6(Es[i] + c * U) == ex[i] for i in idx)
        if k > best[0]:
            best = (k, c)
    return best


def main(lim=300):
    rows = load_rows()
    by = collections.defaultdict(list)
    for r in rows:
        if r["blk"] != "old":
            by[r["blk"]].append(r)
    cs = [i / 8 for i in range(-160, 161)]
    models = {"ctrl exact-E": None, "D-LF1": ("D", "LF1"), "D-LF0": ("D", "LF0"), "E-LF1": ("E", "LF1")}
    for k in sorted(by):
        rs = by[k][:lim]
        ex = [r["excel"] for r in rs]
        ev = [i for i in range(len(rs)) if i % 2 == 0]
        od = [i for i in range(len(rs)) if i % 2 == 1]
        out = [f"{k:18s} a={rs[0]['a']:.6g} b={rs[0]['b']:.6g} n={len(rs)}"]
        for name, spec in models.items():
            if spec is None:
                Es = [
                    R64(
                        mpmath.log(mpf(r["exact"]))
                        if False
                        else mpmath.log(
                            mpf(r["x"]) ** (mpf(r["a"]) - 1)
                            * (1 - mpf(r["x"])) ** (mpf(r["b"]) - 1)
                            / mpmath.beta(mpf(r["a"]), mpf(r["b"]))
                        )
                    )
                    for r in rs
                ]
            else:
                A = Arith(spec[0], "xl", "cr", "lnG")
                Es = [loader_E(A, r["x"], r["a"], r["b"], spec[1]) for r in rs]
            k0 = sum(exp_v6(Es[i]) == ex[i] for i in range(len(rs)))
            kt, c = best_c(Es, ex, ev, cs)
            kh = sum(exp_v6(Es[i] + c * U) == ex[i] for i in od)
            out.append(
                f"   {name:13s} c=0: {k0:3d}/{len(rs)}   best c={c:+.3f}: tune {kt}/{len(ev)} hold-out {kh}/{len(od)}"
            )
        print("\n".join(out))
        sys.stdout.flush()


if __name__ == "__main__":
    main(int(sys.argv[1]) if len(sys.argv) > 1 else 300)
