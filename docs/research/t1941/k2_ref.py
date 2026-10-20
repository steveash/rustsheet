# K2 (GAMMALN on (2,3]) reference model (W2 + reviewer 3, 2026-10-20 07:00 UTC). Not bit-exact.
# CAUTION (07:20): reviewer 3's near-2 ladder test (139k points, x-2 < 5e-7) favours ALL-STORED x87 Horner over the
# 80-bit-head mask used below (grid_a stored 0.9953 vs one extended head sum 0.9685); the mask's +1% on [2,3] may be
# compensating coefficient error. Structure under re-test (far-only refit scored on the near-2 ladders).
# Plain x87 raw-x Horner N(9)/D(8) with plain double coefficients; the HEAD (N stages 1..3, D stages 1..2) is kept in
# 80-bit registers and every op from the hand-off product on (N: P4, D: P3) is stored to double (x87 double rounding);
# y = fl(fl(x-2) * fl(N/D)). Only p1..p5, q0..q4 differ from grid_a (review3/spec.py): grid_a's head was off by a large
# near-null move along c*g(x), g a cubic ~ 1/x (c_N = -3128*2^-54, c_D = +552*2^-54), with p5 -= c_N, q4 -= c_D.
# Coefficients: reviewer-3 basin refit by W2 under the head-80 mask (alternative W2 basin: p4 ...d9003, 0.9522 clusters).
# Scores: 1,024 dense clusters 0.9540 (bias rms 0.039), 14 long runs 0.9507 (grid_a 0.757 / 0.739). numpy longdouble = x87 80-bit.
import sys
import numpy as np
sys.path.insert(0, 'docs/research/t1941/review3')
from spec import load

LD = np.longdouble
P = [float.fromhex(h) for h in ['0x1.5717994ef45d1p-51', '0x1.5e5b738276e47p-16', '0x1.0ee2f11891885p-9',
                                '0x1.572bf86841a77p-5', '0x1.09fd1ad1d8fecp-2', '0x1.f0a8c053986d7p-2',
                                '-0x1.08305b47998b9p-6', '-0x1.0ae34eb3cbad6p-1', '-0x1.de852aee8c153p-3',
                                '-0x1.22260ed97843dp-6']]
Q = [float.fromhex(h) for h in ['0x1.03cadf2cc53b6p-18', '0x1.52fc9914bdd79p-11', '0x1.5303658ef4484p-6',
                                '0x1.bb43b464d3964p-3', '0x1.c515177eb1641p-1', '0x1.7b39fc9194438p+0',
                                '0x1.eddf2040d2bc2p-1', '0x1.9b252b2fab347p-3', '0x1.00000000000eap-7']]

def sd(v):
    return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)

def chain(c, x, K):
    """Head (stages 1..K-1) in x87 80-bit registers; stored double Horner from stage K's product."""
    s = LD(c[0]) + 0 * x
    for i in range(1, len(c)):
        s = s * x + LD(c[i]) if i < K else sd(sd(s * x) + LD(c[i]))
    return s

def k2(x):
    x = np.asarray(x).astype(LD)
    return sd(sd(x - 2) * sd(chain(P, x, 4) / chain(Q, x, 3))).astype(np.float64)

if __name__ == '__main__':
    rates = []
    for case in sys.argv[1:]:
        X, Y = load(case)
        r = float(np.mean(k2(X) == Y)); rates.append(r)
        print(case, len(X), round(r, 4))
    if rates:
        print('mean', round(float(np.mean(rates)), 4))
