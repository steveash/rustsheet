"""Positive control 2: the port's Excel density (Kernel.density = w1's M1) must reproduce w1's published scores
(beta_exact pdf: 685/1,838/1,926 of 2,035; probe1: 10,301/14,827/15,502 of 16,000), and the mpmath EXP/LN models
must equal the engine harness on random inputs."""
import math
import random

import bratio_excel as bx
import score_lib as sl

random.seed(7)
xs = [random.uniform(-700, 700) for _ in range(1500)] + [random.uniform(-3, 3) for _ in range(1500)]
h = bx.em.excel("EXP", xs)
print("EXP model vs harness:", sum(bx.EXP(x) == v for x, v in zip(xs, h)), "/", len(xs))
ls = [math.exp(random.uniform(-700, 700)) for _ in range(1500)] + [random.uniform(0, 2) for _ in range(1500)]
h = bx.em.excel("LN", ls)
print("LN model vs harness:", sum(bx.LN(x) == v for x, v in zip(ls, h)), "/", len(ls))

k = bx.Kernel(bx.BEST)
old = bx.W1.load_pdf()
probe = bx.W1.load_probe("probe1")
for name, rows in (("beta_exact pdf", old), ("w1 probe1", probe)):
    for _ in range(3):
        vals = [k.density(r["x"], r["a"], r["b"]) for r in rows]
        if not bx._GL_MISSING:
            break
        bx.fetch_missing()
    m = [v for v, _ in vals]
    mod = [i for i, (_, f) in enumerate(vals) if f]
    print(name, "all", sl.tally(zip(m, [r["excel"] for r in rows])), "CR", sl.tally((r["exact"], r["excel"]) for r in rows))
    print("   modelled (min<=1)", sl.tally((m[i], rows[i]["excel"]) for i in mod), "CR", sl.tally((rows[i]["exact"], rows[i]["excel"]) for i in mod))
