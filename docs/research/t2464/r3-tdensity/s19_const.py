"""LT1's fitted per-df constant offsets (lt1_offsets.json + hold-out even-half fits) against the error of candidate
bd0(n/2, (n+1)/2) series arithmetics (r3lib.bd0 knobs) and stirlerr: predicted offset = -(bd0_model - bd0_exact)
(units 2^-53).  usage: s19_const.py"""
import itertools

import mpmath
import numpy as np

import tdlib as TD

L = TD.LDB
U = 2.0**-53
OFF = {3: 2.481, 4: -1.613, 5: 1.697, 6: -0.326, 7: -0.009, 8: 0.837, 10: 1.525, 12: -0.398, 20: -0.110, 30: -0.756}
ns = sorted(OFF)
y = np.array([OFF[n] for n in ns])
res = []
knobs = {"v": "xe", "s": "xe", "ej": "xe", "t": "xe", "v2": "xe", "ph": [0, 1], "coef": ["div", "mul"],
         "s0": ["dv", "dd", "ejd"]}
for combo in itertools.product(*knobs.values()):
    o = dict(zip(knobs, combo))
    pred = []
    for n in ns:
        k, M = n / 2, (n + 1) / 2
        b, _ = TD.R3.bd0(L(k), L(M), o)
        ex = TD.bd0_exact(k, M)
        pred.append(-float((mpmath.mpf(float(b)) if False else TD.R3.frac(b) - 0) and 0) if False else
                    -float((mpmath.mpf(TD.R3.frac(b).numerator) / TD.R3.frac(b).denominator - ex) / U))
    pred = np.array(pred)
    res.append((np.sqrt(np.mean((y - pred) ** 2)), o, pred))
res.sort(key=lambda r: r[0])
print("offsets:", " ".join(f"{n}:{v:+.2f}" for n, v in OFF.items()))
for rms, o, p in res[:6]:
    print(f"rms {rms:.2f} {o} pred " + " ".join(f"{v:+.2f}" for v in p))
print("worst", round(res[-1][0], 2), "; rms of offsets themselves", round(float(np.sqrt(np.mean(y ** 2))), 2))
