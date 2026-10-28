"""Ladder covariance of the N top in scan2.py / apply_top.py units (T-2122 run 5, HANDOFF2 17.4): with the rows a
re-read keeps (as k3_null.py) and read noise SIGMA, cov(dN4..dN9) = (M^T M)^-1 SIGMA^2 in ulps; mapped to
Delta S_7(x) = (a + b dx + c dx^2) u, dx = x - 1.1 (u = ulp of S_7 at 1.1): c = dN9/u, b = dN8/u + 2.2 c, all absolute.
usage: k3_topbox.py LOG BASE   env MINRUNG (10), ZERO (500), SIGMA (100)"""
import math
import os
import re
import sys

import numpy as np


def ld(f):
    l = open(f).read().split("\n")
    return [np.array([float.fromhex(v) for v in l[i].split()]) for i in (0, 1)]


MINRUNG, ZERO, SIGMA = int(os.environ.get("MINRUNG", 10)), float(os.environ.get("ZERO", 500)), float(os.environ.get("SIGMA", 100))
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


M = np.array([[(k - j) * x ** (k - j - 1) * np.spacing(abs(N[k])) / math.ulp(abs(sval(j, x))) if k > j else 0.0 for k in ks] for j, x in rows])
cov = np.linalg.inv(M.T @ M) * SIGMA**2
sd = np.sqrt(np.diag(cov))
print("1 sigma (ulps): " + " ".join("N%d %.3g" % (k, v) for k, v in zip(ks, sd)))
u = math.ulp(abs(sval(7, 1.1)))
T = np.zeros((2, len(ks)))  # rows: b, c per ulp of each coefficient
T[1, ks.index(9)] = np.spacing(abs(N[9])) / u
T[0, ks.index(8)] = np.spacing(abs(N[8])) / u
T[0] += 2.2 * T[1]
cb = T @ cov @ T.T
print("scan2 units: sigma_b %.3g, sigma_c %.3g, corr %.3f; a 3-sigma box is |b| <= %.3g, |c| <= %.3g" % (math.sqrt(cb[0, 0]), math.sqrt(cb[1, 1]), cb[0, 1] / math.sqrt(cb[0, 0] * cb[1, 1]), 3 * math.sqrt(cb[0, 0]), 3 * math.sqrt(cb[1, 1])))
print("exact-rate tolerance in the same units (N9 +-400, N8 +-10 ulps): c %.3g, b %.3g" % (400 * T[1, ks.index(9)], 10 * T[0, ks.index(8)]))
