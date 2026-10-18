"""Write run files from the kmyst-erfc-* oracle sidecars (T-1778 round 3, erfc-probes).

Usage: python3 extract.py [OUTDIR]   (reads layout.json written by gen_cases.py)

Per case: OUTDIR/<case>.txt with lines `col run x value`, where col is the sheet column letter of the
formula/constant (B, C, ...), run is the run id from the README, x is column A, and value the oracle result;
x and value are float.hex(); a non-numeric result is written as `!<type>:<text>` (e.g. an error).
Combined files in the round-2 `run x result` format:
  er.txt  ERFC.PRECISE(x)            runs 0-59  (kmyst-erfc-runs1..4)
  ee.txt  EXP(-(x*x))                runs 0-59  (same x)
  cr.txt  CHISQ.DIST.RT(2z,1)        runs 100-129 (kmyst-erfc-chiruns1..3; x column is z)
  cs.txt  ERFC.PRECISE(SQRT(z))      runs 100-129
  cx.txt  EXP(-z)                    runs 100-129
  cq.txt  SQRT(z)                    runs 100-129
"""
import bisect, json, os, sys
CASES = '/home/user/rustsheet/eval/suites/probes-numeric/cases'
HERE = os.path.dirname(os.path.abspath(__file__))
OUT = sys.argv[1] if len(sys.argv) > 1 else HERE
layout = json.load(open(os.path.join(HERE, 'layout.json')))

def fmt(v):
    if v is None:
        return '!missing'
    if v.get('t') == 'n' and isinstance(v.get('v'), (int, float)):
        return float(v['v']).hex()
    return '!%s:%s' % (v.get('t'), v.get('v'))

data = {}
for name, lay in layout.items():
    case = json.load(open(f'{CASES}/{name}.case.json'))['sheets'][0]['cells']
    orc_path = f'{CASES}/{name}.oracle.json'
    if not os.path.exists(orc_path):
        print('no sidecar', name); continue
    orc = json.load(open(orc_path))['sheets']['Calc']
    starts = [r[1] for r in lay['runs']]; ids = [r[0] for r in lay['runs']]
    nrows = sum(1 for k in case if k.startswith('A') and k[1:].isdigit())
    lines = []; per = {}
    for row in range(1, nrows + 1):
        x = float(case[f'A{row}'])
        run = ids[bisect.bisect_right(starts, row) - 1] if starts else 0
        for col in lay['cols']:
            c = case.get(f'{col}{row}')
            if isinstance(c, (int, float)):
                val = float(c).hex()
            else:
                val = fmt(orc.get(f'{col}{row}'))
            lines.append(f'{col} {run} {x.hex()} {val}')
            per.setdefault(col, []).append(f'{run} {x.hex()} {val}')
    open(os.path.join(OUT, name + '.txt'), 'w').write('\n'.join(lines) + '\n')
    data[name] = per
    print(name, len(lines))

def combo(fname, names, col):
    ls = [l for n in names if n in data for l in data[n][col]]
    open(os.path.join(OUT, fname), 'w').write('\n'.join(ls) + '\n'); print(fname, len(ls))

R = [f'kmyst-erfc-runs{i}' for i in range(1, 5)]; C = [f'kmyst-erfc-chiruns{i}' for i in range(1, 4)]
combo('er.txt', R, 'B'); combo('ee.txt', R, 'C')
combo('cr.txt', C, 'B'); combo('cs.txt', C, 'C'); combo('cx.txt', C, 'D'); combo('cq.txt', C, 'E')
