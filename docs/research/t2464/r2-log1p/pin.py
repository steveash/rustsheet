"""Pin Excel's stored l = ln(1-p) per probe row by inverting BINOM.DIST(0,2^k,p) = EXP(2^k * l).
Writes <name>.pins.tsv: p, n, k_ulps (l - CR log1p(-p) in ulps, or 'none'/'multi'), l.  Usage: pin.py name..."""
import math
import sys

import make_probes as MP
import r2lib as R


def pin(name):
    ps = MP.sets(name)
    o = R.read_oracle(name)
    ns = [R.binom_n(p) for p in ps]
    bad_echo = sum(o.get(f"A{i + 1}") != p for i, p in enumerate(ps))
    vals = [o.get(f"B{i + 1}") for i in range(len(ps))]
    hits = R.invert_binom(ps, ns, vals)
    rows, hist = [], {}
    for p, n, h in zip(ps, ns, hits):
        key = h[0] if len(h) == 1 else ("none" if not h else "multi")
        hist[key] = hist.get(key, 0) + 1
        l = R.nb(R.cr_log1p(-p), h[0]) if len(h) == 1 else math.nan
        rows.append((p, n, key, l))
    with open(R.HERE / f"{name}.pins.tsv", "w") as f:
        for r in rows:
            f.write(f"{r[0]!r}\t{r[1]}\t{r[2]}\t{r[3]!r}\n")
    print(name, "rows", len(ps), "A echo mismatches", bad_echo, "offset histogram", dict(sorted(hist.items(), key=str)))
    return rows


def load(name):
    out = []
    for line in open(R.HERE / f"{name}.pins.tsv"):
        p, n, k, l = line.split("\t")
        out.append((float(p), int(n), k, float(l)))
    return out


if __name__ == "__main__":
    for nm in sys.argv[1:]:
        pin(nm)
