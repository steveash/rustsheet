"""Follow-up probe kmyst-erfc-stepmicro (T-1778 round 3, erfc-probes): 800-point grids (step 2E-5) across five strong steps."""
import json, math
OUT = '/home/user/rustsheet/eval/suites/probes-numeric/cases'
PHI = (math.sqrt(5) - 1) / 2
C = [9.0485, 9.6465, 19.3655, 8.0445, 4.1150]
rows = []; desc = []; runs = []
for k, c in enumerate(C):
    desc.append(f'run={k} c={c} row={len(rows) + 1}'); runs.append((k, len(rows) + 1, 800, f'c={c}', c - 0.008))
    rows += [c - 0.008 + 2e-5 * j + PHI * 1e-9 for j in range(800)]
cells = {}
for i, x in enumerate(rows, 1):
    cells[f'A{i}'] = x; cells[f'B{i}'] = f'=ERFC.PRECISE(A{i})'; cells[f'C{i}'] = f'=EXP(-(A{i}*A{i}))'
case = {'format': 'rustsheet-case/v1', 'tags': ['M9', 'probe', 'numeric', 'erf'], 'expected': {},
        'description': 'Kernel-mystery probe (T-1778 round 3, erfc-probes): fine grids (800 points, step 2E-5, x = c - 0.008 + 2E-5*j + 6.18E-10) across five strong steps of the ERFC relative error located in kmyst-erfc-stepmap1/2, to test whether they are true discontinuities and pin their positions to about 1E-4. A = x, B = ERFC.PRECISE(x), C = EXP(-(x*x)). Runs: ' + '; '.join(desc),
        'sheets': [{'name': 'Calc', 'cells': cells}]}
json.dump(case, open(f'{OUT}/kmyst-erfc-stepmicro.case.json', 'w'), indent=1)
lay = json.load(open('layout.json')); lay['kmyst-erfc-stepmicro'] = {'cols': ['B', 'C'], 'runs': runs}
json.dump(lay, open('layout.json', 'w'), indent=1); print(len(cells))
