"""Control: vlib.exp_v6 on doubles against the engine's bit-exact Excel EXP (excelmath), 4,000 random arguments."""
import random

import vlib as V

rnd = random.Random(7)
xs = [rnd.uniform(-700, 700) for _ in range(2000)] + [rnd.uniform(-1, 1) for _ in range(2000)]
ref = V.XM.excel("EXP", xs)
bad = sum(V.fl53(V.exp_v6(x)) != r for x, r in zip(xs, ref))
print(f"exp_v6 vs engine EXP: {bad} mismatches of {len(xs)}")
