"""T-3546: writes probes-numeric/t3546-im-hyperbolic-log-exp-fresh-sweep.case.json, the log form's fresh sweep
(decision 0188 R4.2). Run from the repo root: python3 docs/research/t3546/gen_sweep.py"""

import json,random
rng=random.Random(35461)
cells={}
for r in range(1,151):
    sign=rng.choice([-1,1])
    ax=rng.uniform(708.0,710.5) if sign<0 else rng.uniform(709.0,710.5)
    y=rng.uniform(0.02,1.55)*rng.choice([1,1,1,-1])  # mostly both factors positive, so digits show
    x=sign*ax
    z=f"{x:.15g}{y:+.15g}i"
    cells[f"A{r}"]=f'=IMSINH("{z}")'; cells[f"B{r}"]=f'=IMCOSH("{z}")'
json.dump({"format":"rustsheet-case/v1","description":"T-3546: fresh sweep after freezing the log form's EXP (decision 0188 R4): x < 0 with |x| uniform in [708, 710.5] or x > 0 in [709, 710.5], y in +-[0.02, 1.55] (3 in 4 positive); A IMSINH(x+yi), B IMCOSH(x+yi). Generator: random.Random(35461), see the T-3546 Outcome.","tags":["M9","engineering","probe","T-3546"],"expected":{},"sheets":[{"name":"S","cells":cells}]},open("eval/suites/probes-numeric/cases/t3546-im-hyperbolic-log-exp-fresh-sweep.case.json","w"),indent=2)
