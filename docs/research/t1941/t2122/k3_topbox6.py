"""Ladder covariance of the N top mapped to the first stored stage below an unstored top (T-2122 run 6, HANDOFF2 18):
as k3_topbox.py, but for Delta S_6(x) = (a + b dx + c dx^2 + d dx^3) u, dx = x - 1.1 (u = ulp of S_6 at 1.1), the unknown
of a scan3.py-style J=6 scan (S_6 = N6 + x S_7): d = dN9/u, c = (dN8 + 3.3 dN9)/u, b = (dN7 + 2.2 dN8 + 3.63 dN9)/u.
Prints the 3-sigma box and its size in scan2/scan3 grid points (step CSTEP in c, DSTEP in d).
usage: k3_topbox6.py LOG BASE   env MINRUNG (10), ZERO (500), SIGMA (100), CSTEP (0.5), DSTEP (0.5)"""
import math
import os
import re
import sys

import numpy as np


def ld(f):
    l = open(f).read().split("\n")
    return [np.array([float.fromhex(v) for v in l[i].split()]) for i in (0, 1)]


MINRUNG, ZERO, SIGMA = int(os.environ.get("MINRUNG", 10)), float(os.environ.get("ZERO", 500)), float(os.environ.get("SIGMA", 100))
CSTEP, DSTEP = float(os.environ.get("CSTEP", 0.5)), float(os.environ.get("DSTEP", 0.5))
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
print("%d rows at zero; 1 sigma (ulps): " % len(rows) + " ".join("N%d %.3g" % (k, v) for k, v in zip(ks, np.sqrt(np.diag(cov)))))
u = math.ulp(abs(sval(6, 1.1)))
sp = {k: float(np.spacing(abs(N[k]))) / u for k in ks}
T = np.zeros((3, len(ks)))  # rows: b, c, d per ulp of each coefficient
T[2, ks.index(9)] = sp[9]
T[1, ks.index(8)], T[1, ks.index(9)] = sp[8], 3.3 * sp[9]
T[0, ks.index(7)], T[0, ks.index(8)], T[0, ks.index(9)] = sp[7], 2.2 * sp[8], 3.63 * sp[9]
cb = T @ cov @ T.T
sd = np.sqrt(np.diag(cb))
cr = cb / np.outer(sd, sd)
print("scan3 units (J=6): sigma_b %.3g, sigma_c %.3g, sigma_d %.3g; corr bc %.3f bd %.3f cd %.3f" % (*sd, cr[0, 1], cr[0, 2], cr[1, 2]))
print("3-sigma box: |b| <= %.3g, |c| <= %.3g, |d| <= %.3g" % tuple(3 * sd))
nc, nd = 2 * 3 * sd[1] / CSTEP + 1, 2 * 3 * sd[2] / DSTEP + 1
print("grid at CSTEP %g, DSTEP %g: %.3g c x %.3g d = %.3g (c, d) pairs" % (CSTEP, DSTEP, nc, nd, nc * nd))
# conditional box: d fixed, c's spread given d (the c-d correlation shrinks the c range per d)
sc_d = sd[1] * math.sqrt(max(1 - cr[1, 2] ** 2, 0.0))
print("c given d: sigma %.3g -> %.3g (c, d) pairs along the c-d ridge" % (sc_d, (2 * 3 * sc_d / CSTEP + 1) * nd))
print("exact-rate tolerance (N9 +-400, N8 +-10, N7 +-1 ulps): d %.3g, c %.3g, b %.3g" % (400 * sp[9], 10 * sp[8], 1 * sp[7]))
