"""T-3571: the fit lab's hash hold-out split (decision 0188 R4.1, T-3360) on the r5-bfrac probes, scored with
score_final.table. Uses the fit lab's own `is_holdout` (tools/src/rustsheet_tools/fitlab/holdout.py) with the cell's
case id `probes-numeric/<case>` and key `Calc!A<k>` (the cdf cell); margin DEFAULT_MARGIN (5 points)."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[4] / 'tools' / 'src'))
from rustsheet_tools.fitlab.holdout import DEFAULT_MARGIN, is_holdout  # noqa: E402
from rustsheet_tools.fitlab.sweep import Cell  # noqa: E402
from runs import load  # noqa: E402
from score_final import table  # noqa: E402

def held(case, k):
    return is_holdout(Cell(case=f'probes-numeric/{case}', sheet='Calc', address=f'A{k}', formula='', args=(), expected=0.0))

CASES = ['t3571-r5-bfrac-tune', 't3571-r5-bfrac-hold', 't3571-r5-bfrac-fresh']
rows = {c: load(c) for c in CASES}
print(f'margin {DEFAULT_MARGIN:.0%}')
for label, cs in (('tune+hold', CASES[:2]), ('tune only', CASES[:1]), ('all three', CASES)):
    tr = [r for c in cs for r in rows[c] if not held(c, r['k'])]
    te = [r for c in cs for r in rows[c] if held(c, r['k'])]
    T, E = table(tr), table(te)
    print(f'{label}: train {len(tr)}, test {len(te)}')
    for nm in T:
        print(f'  {nm:14s} train {T[nm][0]}/{T[nm][1]}/{T[nm][2]} ({T[nm][0] / len(tr):.1%})  '
              f'test {E[nm][0]}/{E[nm][1]}/{E[nm][2]} ({E[nm][0] / len(te):.1%})')
