"""Do Excel's per-(shape, side) constant offsets (s4_kscan*.out, M-xd base) follow r3-loader-const's stirlerr-error
table e(v) (grid_e.tsv, gauge alpha v + gamma)?  pred = e(h) - e(a) - e(b) (gauge-free up to a constant), fitted
constant c0. Prints per group and the RMS against the null (one constant).  Usage: s18_etab.py MINC file [file...]"""
import re
import sys

import numpy as np

import slib as L

MINC = int(sys.argv[1])
E = {}
for line in (L.T / "r3-loader-const" / "grid_e.tsv").read_text().splitlines()[1:]:
    v, e = line.split("\t")
    E[float(v)] = float(e)
obs = []
for f in sys.argv[2:]:
    for line in open(L.HERE / f):
        m = re.match(r"\((\d+),(\d+)\) s(\d) n\s+(\d+) best\s+(\d+) delta \[\s*([-+.\d]+),\s*([-+.\d]+)\]", line)
        if m and int(m[5]) >= MINC:
            a, b = float(m[1]) / 2, float(m[2]) / 2
            if a + b in E and a in E and b in E:
                obs.append((float(m[1]), float(m[2]), int(m[3]), 0.5 * (float(m[6]) + float(m[7])),
                            E[a + b] - E[a] - E[b]))
o = np.array([x[3] for x in obs])
p = np.array([x[4] for x in obs])
for sgn in (+1, -1):
    c0 = float(np.mean(o - sgn * p))
    res = o - sgn * p - c0
    print(f"sign {sgn:+d}: c0 {c0:+.2f} RMS {np.sqrt((res**2).mean()):.2f} | null RMS {o.std():.2f} | corr {np.corrcoef(o, p)[0, 1]:+.2f} n {len(o)}")
for m, n, s, d, pr in obs:
    print(f"({m:g},{n:g}) s{s} obs {d:+6.2f} e(h)-e(a)-e(b) {pr:+7.2f}")
