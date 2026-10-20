import json, glob, os, sys
import numpy as np
lo, hi, out = float(sys.argv[1]), float(sys.argv[2]), sys.argv[3]; side = sys.argv[4] if len(sys.argv) > 4 else 'right'
X, B, C = [], [], []
for f in sorted(glob.glob('eval/suites/probes-numeric/cases/kmyst-gam*.case.json')):
    o = f.replace('.case.json', '.oracle.json')
    if not os.path.exists(o): continue
    cells = json.load(open(f))['sheets'][0]['cells']
    if 'GAMMALN' not in str(cells.get('B1', '')).upper(): continue
    v = json.load(open(o))['sheets']['Calc']; nm = os.path.basename(f)[:-10]; r = 1
    while f"A{r}" in cells:
        y = v.get(f"B{r}", {}).get('v')
        try: xv = float(cells[f'A{r}'])
        except (TypeError, ValueError): r += 1; continue
        inr = (lo < xv <= hi) if side == 'right' else (lo <= xv < hi)
        if isinstance(y, (int, float)) and inr: X.append(xv); B.append(float(y)); C.append(nm)
        r += 1
x = np.array(X); b = np.array(B); c = np.array(C); x, ui = np.unique(x, return_index=True)
np.savez(out, x=x, b=b[ui], c=c[ui]); print('%d points from %d cases' % (len(x), len(set(c))))
