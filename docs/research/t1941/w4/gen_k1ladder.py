# T-1941 W4 K1 dip phase-unwrap ladders: kmyst-gam-k1l<a..f><kk>, one run of 8,192 consecutive doubles each.
# Anchors (non-dyadic, inside the GAMMA [1,2) dip 1.80-1.97): a 1.8013, b 1.8417, c 1.8713, d 1.8911, e 1.9219, f 1.9613.
# Per anchor, starts at fl(anchor + 2^-k) for k = 10..21 (kk = 10..21) and at the anchor itself (kk = 00): stepwise
# unwrapping of the D P2 (S3*y) line phase into Excel's S3(y). B = GAMMA(A) only.
# usage (repo root): python3 docs/research/t1941/w4/gen_k1ladder.py eval/suites/probes-numeric/cases
import json, math, sys, os
D = sys.argv[1]
ANCH = {'a': 1.8013, 'b': 1.8417, 'c': 1.8713, 'd': 1.8911, 'e': 1.9219, 'f': 1.9613}
n = 0
for tag, a in ANCH.items():
    for k in [0] + list(range(10, 22)):
        x0 = a if k == 0 else a + 2.0**-k
        p = os.path.join(D, f'kmyst-gam-k1l{tag}{k:02d}.case.json')
        if os.path.exists(p): continue   # already generated
        cells = {}; x = x0
        for r in range(1, 8193):
            cells[f'A{r}'] = x; cells[f'B{r}'] = f'=GAMMA(A{r})'; x = math.nextafter(x, math.inf)
        desc = (f'Kernel-mystery probe (T-1941 W4): K1 GAMMA dip phase-unwrap ladder, anchor {a} ' +
                ('(base run)' if k == 0 else f'+ 2^-{k}') + f', 8,192 consecutive doubles from {x0!r}. B = GAMMA(x).')
        json.dump({'format': 'rustsheet-case/v1', 'description': desc, 'tags': ['M9', 'probe', 'numeric', 'gamma'], 'expected': {},
                   'sheets': [{'name': 'Calc', 'cells': cells}]}, open(p, 'w'), indent=1)
        n += 1
print(n, 'cases written')
