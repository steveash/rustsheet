"""a2: which (k, n, n-k) argument form fits Excel's x-dependence and makes the shape offset smooth in ulp sweeps.

Per cell: Ei = mid of the extended-E interval (EXP_V6 inverted), Xi = xpart_v(form), lp_i = exact ln pdf.
- within-shape spread of Ei - Xi (a correct X makes it constant up to the interval width, ~0.3-0.6 units sd);
- per shape the offset o = median(Ei - Xi) - Cex, Cex the exact constant at R's stored k, n, nk (r3lib.exact_const),
  i.e. Excel's constant minus the exact one (the fl(a+b) rounding of n is Excel's, so it is not counted as error);
- per sweep base: sd of o across its ulp-neighbour shapes (pieces formed differently show up as jumps).
Units 2^-53 (absolute in E). Usage: a2_forms.py sweep|diverse FORM [FORM...]; writes off_<which>_<form>.tsv.
"""

from __future__ import annotations

import statistics as st
import sys
from collections import defaultdict

import mpmath

import r3lib as R

U = mpmath.mpf(2) ** -53


def lnpdf(x, a, b):
    A, B, X = mpmath.mpf(a), mpmath.mpf(b), mpmath.mpf(x)
    return (A - 1) * mpmath.log(X) + (B - 1) * mpmath.log1p(-X) - mpmath.log(mpmath.beta(A, B))


_LP: dict = {}
_EI: dict = {}


def cells(rows):
    by = defaultdict(list)
    for r in rows:
        key = (r["x"], r["a"], r["b"])
        if key not in _LP:
            _LP[key] = lnpdf(*key)
            _EI[key] = R.e_interval_ext(r["excel"])
        if _EI[key] is not None:
            by[(r["a"], r["b"])].append((key, (_EI[key][0] + _EI[key][1]) / 2, _LP[key]))
    return by


def offsets(by, form):
    within, off = [], {}
    for sh, cs in by.items():
        ref = R.exact_const(*sh, form)  # subtracted before float() keeps sub-unit precision
        ex = [float((e - R.xpart_v(*key, form) - ref) / U) for key, e, lp in cs]
        m = st.median(ex)
        within += [v - m for v in ex]
        off[sh] = (m, len(cs), st.pstdev(ex))
    return within, off


def base_of(sh):
    for a0, b0 in R.SWEEP_BASES + R.HOLD_BASES:
        if abs(sh[0] - a0) < 1e-9 * a0 and abs(sh[1] - b0) < 1e-9 * b0:
            return (a0, b0)
    return None


def main():
    which = sys.argv[1]
    by = cells(R.load(which))
    for form in sys.argv[2:]:
        within, off = offsets(by, form)
        sw = st.pstdev(within)
        mad = st.median([abs(v) for v in within])
        print(f"form {form}: within-shape sd(E - X) {sw:.2f}, median |dev| {mad:.2f} (cells {len(within)})")
        bases = defaultdict(list)
        for sh, (o, n, s) in off.items():
            bases[base_of(sh)].append(o)
        if which in ("sweep", "hold"):
            print("   base: mean / sd of offset across ulp neighbours")
            for b, os_ in sorted(bases.items(), key=lambda t: str(t[0])):
                print(f"   {b}: {st.mean(os_):+7.2f} / {st.pstdev(os_):5.2f}  (n {len(os_)})")
        with open(R.HERE / f"off_{which}_{form}.tsv", "w") as f:
            f.write("a\tb\tcells\toffset_units\twithin_sd\n")
            for (a, b), (o, n, s) in sorted(off.items()):
                f.write(f"{a!r}\t{b!r}\t{n}\t{o:+.3f}\t{s:.3f}\n")
    R.save_cache()


if __name__ == "__main__":
    main()
