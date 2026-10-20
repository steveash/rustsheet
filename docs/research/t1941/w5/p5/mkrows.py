"""Build the K3 point set (T-1941 W5 part 5): kmyst-gam rows whose column B really is GAMMALN.PRECISE (the shared
extractor assumes it; q501/q502 have B = GAMMA), x in [0.7, 1.5), unique x. usage: mkrows.py OUT.npz"""
import json, glob, os, sys
import numpy as np
X, B, C = [], [], []
for f in sorted(glob.glob('eval/suites/probes-numeric/cases/kmyst-gam*.case.json')):
    o = f.replace('.case.json', '.oracle.json')
    if not os.path.exists(o):
        continue
    cells = json.load(open(f))['sheets'][0]['cells']
    if 'GAMMALN' not in str(cells.get('B1', '')).upper():
        continue
    v = json.load(open(o))['sheets']['Calc']; nm = os.path.basename(f)[:-10]; r = 1
    while f"A{r}" in cells:
        y = v.get(f"B{r}", {}).get('v'); xv = float(cells[f'A{r}'])
        if isinstance(y, (int, float)) and 0.7 <= xv < 1.5:
            X.append(xv); B.append(float(y)); C.append(nm)
        r += 1
x = np.array(X); b = np.array(B); c = np.array(C)
x, ui = np.unique(x, return_index=True)
np.savez(sys.argv[1], x=x, b=b[ui], c=c[ui]); print('%d points from %d cases' % (len(x), len(set(c))))
