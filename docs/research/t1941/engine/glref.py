"""GAMMALN reference composed from the T-1941 checkers (numpy longdouble = x87 80-bit; sd = store).
K2: k2_ref.py (head-80 mask) and cand9 (stored, review3/spec.py sim); K3: w5/p5/maskmodel.py res (all stored);
(1.5,2]: w6/synth.py simB (all stored, xl_chain_d3ls); x>=8: gammaln_big_check.c; LN: lnmodels.py FYL2X model."""
import sys, numpy as np, mpmath
sys.path.insert(0, sys.argv[1] if len(sys.argv) > 1 else '.')
from rnd import rn53, rn64
LD = np.longdouble
def sd(v): return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)
mpmath.mp.prec = 200
_L2 = rn64(mpmath.log(2))
def ln1(x): return rn53(rn64(_L2 * mpmath.log(mpmath.mpf(float(x)), 2)))
def LN(x): return np.array([ln1(v) for v in np.atleast_1d(x)], dtype=np.float64)
H = lambda s: [float.fromhex(h) for h in s.split()]
P = H('0x1.5717994ef45d1p-51 0x1.5e5b738276e47p-16 0x1.0ee2f11891885p-9 0x1.572bf86841a77p-5 0x1.09fd1ad1d8fecp-2 0x1.f0a8c053986d7p-2 -0x1.08305b47998b9p-6 -0x1.0ae34eb3cbad6p-1 -0x1.de852aee8c153p-3 -0x1.22260ed97843dp-6')
Q = H('0x1.03cadf2cc53b6p-18 0x1.52fc9914bdd79p-11 0x1.5303658ef4484p-6 0x1.bb43b464d3964p-3 0x1.c515177eb1641p-1 0x1.7b39fc9194438p+0 0x1.eddf2040d2bc2p-1 0x1.9b252b2fab347p-3 0x1.00000000000eap-7')
P9 = [P[0]] + H('0x1.5e5b738276791p-16 0x1.0ee2f118918cdp-9 0x1.572bf86841a71p-5 0x1.09fd1ad1d8fecp-2 0x1.f0a8c053986d7p-2') + P[6:]
Q9 = H('0x1.03cadf2cc5614p-18 0x1.52fc9914bdd53p-11 0x1.5303658ef4486p-6 0x1.bb43b464d3964p-3 0x1.c515177eb1641p-1') + Q[5:]
K3 = [H(l) for l in open(sys.argv[1] + '/ss_fc.txt') if l.strip()]
W6 = {l.split()[0]: H(' '.join(l.split()[1:])) for l in open(sys.argv[1] + '/xl_chain_d3ls.txt') if l.strip()}
def chain(c, x, K):
    s = LD(c[0]) + 0 * x
    for i in range(1, len(c)):
        s = s * x + LD(c[i]) if i < K else sd(sd(s * x) + LD(c[i]))
    return s
def k2ref(x):
    x = np.asarray(x).astype(LD); return sd(sd(x - 2) * sd(chain(P, x, 4) / chain(Q, x, 3))).astype(np.float64)
def k2c9(x):
    x = np.asarray(x).astype(LD); n = LD(P9[0]) + 0 * x; d = LD(Q9[0]) + 0 * x
    for c in P9[1:]: n = sd(sd(n * x) + LD(c))
    for c in Q9[1:]: d = sd(sd(d * x) + LD(c))
    return sd(sd(x - LD(2)) * sd(n / d)).astype(np.float64)
def asc_chain(C, xl):
    s = LD(C[-1]) + 0 * xl
    for j in range(len(C) - 2, -1, -1): s = sd(sd(s * xl) + LD(C[j]))
    return s
def k3(x):
    xl = np.asarray(x).astype(LD); t = sd(xl - LD(1.0))
    return sd(t * sd(asc_chain(K3[0], xl) / asc_chain(K3[1], xl))).astype(np.float64)
def k15(x):
    xl = np.asarray(x).astype(LD); R = sd(asc_chain(W6['N'], xl) / asc_chain(W6['D'], xl))
    return sd(sd(xl - 1.0) * sd(sd(xl - 2.0) * R)).astype(np.float64)
BC = [1.0/12, -1.0/360, 1.0/1260, -1.0/1680, 1.0/1188, -691.0/360360, 1.0/156, -3617.0/122400, 43867.0/244188, -174611.0/125400]
def stirling(x):
    y = np.asarray(x).astype(LD); corr = LN(x).astype(LD)
    a = sd(y - LD(0.5)); b = sd(a * corr); d = sd(b - y); e = sd(d + LD(0.9189385332046727))
    z = sd(LD(1) / y); z2 = sd(z * z); r = LD(BC[9]) + 0 * y
    for i in range(8, -1, -1): r = sd(sd(r * z2) + LD(BC[i]))
    return sd(e + sd(r * z)).astype(np.float64)
CUT = float(sys.argv[2]) if len(sys.argv) > 2 else 0.0
def k2(t):
    t = np.asarray(t, dtype=np.float64); out = k2ref(t)
    m = (t - 2.0) < CUT
    if m.any(): out[m] = k2c9(t[m])
    return out
def gl_from07(x):
    x = np.asarray(x, dtype=np.float64); out = np.empty_like(x)
    for lo, hi, f in ((0.7, 1.5, k3),):
        pass
    m = x <= 1.5; out[m] = k3(x[m]) if m.any() else out[m]
    m = (x > 1.5) & (x <= 2.0); out[m] = k15(x[m]) if m.any() else out[m]
    m = (x > 2.0) & (x <= 3.0); out[m] = k2(x[m]) if m.any() else out[m]
    m = (x > 3.0) & (x < 8.0)
    if m.any():
        xs = x[m]; t = xs.copy(); steps = np.zeros(len(xs), int)
        while (t >= 3.0).any(): s = t >= 3.0; t[s] -= 1.0; steps[s] += 1
        p = t.astype(LD)
        for i in range(1, 6):
            s = steps > i; p[s] = sd(p[s] * (t[s] + i).astype(LD))
        out[m] = sd(k2(t).astype(LD) + LN(p.astype(np.float64)).astype(LD)).astype(np.float64)
    m = x >= 8.0; out[m] = stirling(x[m]) if m.any() else out[m]
    return out
def gammaln(x):
    x = np.asarray(x, dtype=np.float64); out = np.empty_like(x); low = x < float.fromhex('0x1.6666666666666p-1')
    out[~low] = gl_from07(x[~low])
    if low.any():
        xl = x[low]; up = sd(xl.astype(LD) + LD(1)).astype(np.float64)
        out[low] = sd(gl_from07(up).astype(LD) - LN(xl).astype(LD)).astype(np.float64)
    return out
