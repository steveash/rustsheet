"""Tabulate K3 ladder re-reads (T-2122 run 5, HANDOFF2 17): w5/p5/ladder.py run with a MOVED set as its base reads the
residual slope of each stage (cycles of ulp(S_j) per unit x). A right move leaves every readable row near 0; the
reading does not depend on the low bits of the coefficients below the stage, unlike the refit-and-count score of 15.3.
Rows with fewer than MINRUNG rungs are shown but not counted (they are the ones that mis-unwrap to 1e6-1e8).
usage: k3_reread.py LOG...   env MINRUNG (10), ZERO (500 cycles/x: a row counts as 'at zero' below it), CH (N)"""

import os
import re
import sys

MINRUNG = int(os.environ.get("MINRUNG", 10))
ZERO = float(os.environ.get("ZERO", 500))
CH = os.environ.get("CH", "N")
ROW = re.compile(
    r"\s+([ND])(\d): slope\s+([+-][\d.]+) cycles/x from (\d+) rungs \([\d,]+\)(?: \| truth\s+([+-][\d.]+))?"
)


def parse(path):
    out, a = {}, None
    for line in open(path):
        m = re.match(r"anchor (\w) \(x ([\d.]+)\)", line)
        if m:
            a = m.group(2)
            continue
        m = ROW.match(line)
        if m and a is not None and m.group(1) == CH:
            out[(a, m.group(1) + m.group(2))] = (
                float(m.group(3)),
                int(m.group(4)),
                m.group(5) and float(m.group(5)),
            )
    return out


logs = [parse(p) for p in sys.argv[1:]]
keys = sorted(set().union(*logs), key=lambda k: (k[1], k[0]))
print("%-14s" % "row" + "".join("%26s" % os.path.basename(p)[:24] for p in sys.argv[1:]) + "      truth")
for k in keys:
    cells, truth = [], None
    for lg in logs:
        if k in lg:
            s, n, t = lg[k]
            truth = t if t is not None else truth
            cells.append("%+17.1f (%2d)%s" % (s, n, "*" if n >= MINRUNG and abs(s) < ZERO else " "))
        else:
            cells.append("%23s" % "-")
    print(
        "%-14s" % ("%s@%s" % (k[1], k[0]))
        + "".join("%26s" % c for c in cells)
        + ("  %+10.1f" % truth if truth is not None else "")
    )
for p, lg in zip(sys.argv[1:], logs):
    good = [v for v in lg.values() if v[1] >= MINRUNG]
    z = [v[0] for v in good if abs(v[0]) < ZERO]
    rms = (sum(s * s for s in z) / len(z)) ** 0.5 if z else float("nan")
    print(
        "%-40s %2d rows with >= %d rungs, %2d at zero (|s| < %g), rms of those %.1f cycles/x"
        % (os.path.basename(p), len(good), MINRUNG, len(z), ZERO, rms)
    )
