"""w6-literature H2b: residual structure of Loader-style density variants (a, b > 2), per block.

Exact-match counts are a poor objective far from the truth (playbook section 4), so this reports, per block,
the mean and sd of Excel's relative error against each model (units of 2^-53) and the correlation of
(model - CR) with (Excel - CR). A right structure with a wrong constant shows a small sd and a high correlation.
Control row: the CR value itself (sd of Excel's own noise).
"""

import collections
import itertools
import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from h2_loader import Arith, load_rows  # noqa: E402

U = 2.0**-53


def stats(m, rows):
    d = [(r["excel"] - mm) / r["excel"] / U for mm, r in zip(m, rows) if math.isfinite(mm)]
    e = [(r["excel"] - r["exact"]) / r["excel"] / U for mm, r in zip(m, rows) if math.isfinite(mm)]
    q = [(mm - r["exact"]) / r["excel"] / U for mm, r in zip(m, rows) if math.isfinite(mm)]
    n = len(d)
    mu = sum(d) / n
    sd = math.sqrt(sum((v - mu) ** 2 for v in d) / n)
    me, mq = sum(e) / n, sum(q) / n
    cov = sum((a - me) * (b - mq) for a, b in zip(e, q))
    va = sum((a - me) ** 2 for a in e)
    vb = sum((b - mq) ** 2 for b in q)
    c = cov / math.sqrt(va * vb) if va > 0 and vb > 0 else float("nan")
    return mu, sd, c


def main(lim=200):
    rows = load_rows()
    by = collections.defaultdict(list)
    for r in rows:
        if r["blk"] == "old":
            continue
        by[r["blk"]].append(r)
    blks = sorted(by)
    print("block: CR (mean, sd) | variants (mean, sd, corr)")
    variants = [
        ("D", "LF1", "lnG"),
        ("D", "LF0", "lnG"),
        ("D", "FX", "lnG"),
        ("E", "LF1", "lnG"),
        ("E", "LF0", "lnG"),
        ("E", "FX", "lnG"),
        ("D", "LF1", "GL"),
    ]
    for k in blks:
        rs = by[k][:lim]
        a, b = rs[0]["a"], rs[0]["b"]
        cr = [r["exact"] for r in rs]
        mu, sd, _ = stats(cr, rs)
        parts = [f"{k:20s} a={a:.6g} b={b:.6g} n={len(rs)}  CR ({mu:+.2f}, {sd:.2f})"]
        for mode, lff, lg in variants:
            A = Arith(mode, "xl", "cr", lg)
            m = []
            for r in rs:
                try:
                    m.append(A.dbeta(r["x"], r["a"], r["b"], lff))
                except Exception:  # noqa: BLE001
                    m.append(math.nan)
            mu, sd, c = stats(m, rs)
            parts.append(f"   {mode}-{lff}-{lg}: ({mu:+.2f}, {sd:.2f}, r={c:+.2f})")
        print("\n".join(parts))
        sys.stdout.flush()


if __name__ == "__main__":
    main(int(sys.argv[1]) if len(sys.argv) > 1 else 200)
