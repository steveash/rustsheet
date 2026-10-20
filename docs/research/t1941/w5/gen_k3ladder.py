# T-1941 W5 K3 phase-unwrap ladders: kmyst-gam-k3l<a..f><kk>, one run of 8,192 consecutive doubles each.
# Anchors (non-dyadic): a 0.8013, b 0.9517, c 1.1023, d 1.2519, e 1.3527, f 1.4531. Per anchor, starts at
# fl(anchor + 2^-k) for k = 10..21 (kk = 10..21) and at the anchor itself (kk = 00): stepwise unwrapping of the steep
# per-stage offset laws (w5/p5 README, "Per-stage phase read"). B = GAMMALN.PRECISE(A) only.
# usage (repo root): python3 docs/research/t1941/w5/gen_k3ladder.py eval/suites/probes-numeric/cases
import json, math, sys, os
D = sys.argv[1]
ANCH = {'a': 0.8013, 'b': 0.9517, 'c': 1.1023, 'd': 1.2519, 'e': 1.3527, 'f': 1.4531}
n = 0
for tag, a in ANCH.items():
    for k in [0] + list(range(10, 22)):
        x0 = a if k == 0 else a + 2.0**-k
        p = os.path.join(D, f'kmyst-gam-k3l{tag}{k:02d}.case.json'); assert not os.path.exists(p), p
        cells = {}; x = x0
        for r in range(1, 8193):
            cells[f'A{r}'] = x; cells[f'B{r}'] = f'=GAMMALN.PRECISE(A{r})'; x = math.nextafter(x, math.inf)
        desc = (f'Kernel-mystery probe (T-1941 W5): K3 phase-unwrap ladder, anchor {a} ' +
                ('(base run)' if k == 0 else f'+ 2^-{k}') + f', 8,192 consecutive doubles from {x0!r}. B = GAMMALN.PRECISE(x).')
        json.dump({'format': 'rustsheet-case/v1', 'description': desc, 'tags': ['M9', 'probe', 'numeric', 'gamma'], 'expected': {},
                   'sheets': [{'name': 'Calc', 'cells': cells}]}, open(p, 'w'), indent=1)
        n += 1
print(n, 'cases written')
