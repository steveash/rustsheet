"""Positive control: lib.exp_v6 / lib.ln_x87 (mpmath) vs the engine's EXP/LN models on random doubles."""
import random
from lib import xl, exp_v6, ln_x87
random.seed(1)
xs=[random.uniform(-40,40) for _ in range(3000)]+[random.uniform(-1,1) for _ in range(2000)]
e=xl('EXP',xs); m=[exp_v6(x) for x in xs]
print('EXP match', sum(a==b for a,b in zip(e,m)), len(xs))
ys=[random.uniform(1e-3,3) for _ in range(3000)]+[10**random.uniform(-300,300) for _ in range(1000)]
l=xl('LN',ys); m=[ln_x87(y) for y in ys]
print('LN match', sum(a==b for a,b in zip(l,m)), len(ys))
