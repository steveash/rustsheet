"""Weak directions of the K3 N-chain slope equations (T-2122 run 5, HANDOFF2 17). The ladder rows a re-read keeps
(>= MINRUNG rungs, |slope| < ZERO cycles/x) give M[row, k] = d slope_j(a) / d N_k in cycles/x per ulp of N_k,
k = 4..9. Prints the singular values (in ulps per SIGMA cycles/x of read noise) and directions, and, with a truth
move (control), its projection on each direction: the move the ladder cannot see is what the next scan must search.
usage: k3_null.py LOG BASE [TRUTHSET]   env MINRUNG (10), ZERO (500), SIGMA (100 cycles/x)"""

import math
import os
import re
import sys

import numpy as np


def ld(f):
    l = open(f).read().split("\n")
    return [np.array([float.fromhex(v) for v in l[i].split()]) for i in (0, 1)]


MINRUNG, ZERO, SIGMA = (
    int(os.environ.get("MINRUNG", 10)),
    float(os.environ.get("ZERO", 500)),
    float(os.environ.get("SIGMA", 100)),
)
N, _ = ld(sys.argv[2])
rows, a = [], None
for line in open(sys.argv[1]):
    m = re.match(r"anchor (\w) \(x ([\d.]+)\)", line)
    if m:
        a = float(m.group(2))
        continue
    m = re.match(r"\s+N(\d): slope\s+([+-][\d.]+) cycles/x from (\d+) rungs", line)
    if m and a is not None and int(m.group(3)) >= MINRUNG and abs(float(m.group(2))) < ZERO:
        rows.append((int(m.group(1)), a))
ks = list(range(4, len(N)))


def sval(j, x):
    return sum(float(N[k]) * x ** (k - j) for k in range(j, len(N)))


M = np.array(
    [
        [
            (k - j) * x ** (k - j - 1) * np.spacing(abs(N[k])) / math.ulp(abs(sval(j, x))) if k > j else 0.0
            for k in ks
        ]
        for j, x in rows
    ]
)
print("%d rows: %s" % (len(rows), " ".join("N%d@%.4f" % r for r in rows)))
U, s, Vt = np.linalg.svd(M / SIGMA, full_matrices=False)
for i in range(len(s)):
    print(
        "dir %d: %.3g ulps of move per unit of read noise; " % (i, 1 / s[i])
        + " ".join("N%d %+.4f" % (k, v) for k, v in zip(ks, Vt[i]))
    )
if len(sys.argv) > 3:
    Nt, _ = ld(sys.argv[3])
    mv = np.array([(Nt[k] - N[k]) / np.spacing(abs(N[k])) for k in ks])
    print("truth move (ulps): " + " ".join("N%d %+.0f" % (k, v) for k, v in zip(ks, mv)))
    print("projections (ulps): " + " ".join("dir%d %+.4g" % (i, float(Vt[i] @ mv)) for i in range(len(s))))
    print("truth slope residual on the rows (cycles/x): " + " ".join("%+.0f" % v for v in M @ mv))
