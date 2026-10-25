"""Density C = BETA.DIST(p,1,n+1,FALSE) modelled as EXP(n*l + LN(n+1)) (sum extended or stored, EXP V6), with l
the BINOM pin; then, where BINOM is #NUM! (n >= 2^31), pins l from C alone with that model.
Usage: dens_model.py"""
import math

import make_probes as MP
import pin
import r2lib as R
from lib import exp_v6  # w1 mpmath EXP V6 (accepts extended input)

fl64 = R.excelmath.fl64


def model(n, l, mode):
    lnb = R.xl("LN", [float(n + 1)])[0]
    e = R.mpf(n) * R.mpf(l) + R.mpf(lnb)
    e = fl64(e) if mode == "ext" else R.mpf(R.excelmath.fl53(fl64(e)))
    return exp_v6(e)


def main():
    cnt = {"ext": [0, 0], "st": [0, 0]}
    tiny = []
    for nm in ("runs", "grid", "hold"):
        o = R.read_oracle(nm)
        for i, (p, n, k, l) in enumerate(pin.load(nm)):
            c = o.get(f"C{i + 1}")
            if k in ("none", "multi"):
                tiny.append((p, n, c))
                continue
            for m in cnt:
                cnt[m][0] += model(n, l, m) == c
                cnt[m][1] += 1
    print("density model on BINOM-pinned rows:", cnt)
    # pin tiny-p rows from the density alone (ext model)
    hist = {}
    rows = []
    for p, n, c in tiny:
        l0 = R.cr_log1p(-p)
        hits = [j for j in range(-6, 7) if model(n, R.nb(l0, j), "ext") == c]
        key = hits[0] if len(hits) == 1 else ("none" if not hits else "multi")
        hist[key] = hist.get(key, 0) + 1
        rows.append((p, n, key, R.nb(l0, hits[0]) if len(hits) == 1 else math.nan))
    print("tiny-p rows pinned from density:", hist)
    with open(R.HERE / "tiny.pins.tsv", "w") as f:
        for r in rows:
            f.write(f"{r[0]!r}\t{r[1]}\t{r[2]}\t{r[3]!r}\n")


if __name__ == "__main__":
    main()
