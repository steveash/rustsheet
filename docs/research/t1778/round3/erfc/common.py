import mpmath as mp, math, json
mp.mp.prec = 140
CASES = '/home/user/rustsheet/eval/suites/probes-numeric/cases'
EPS = 2.0 ** -53

def load(name):
    """rows: list of dicts col->value (float or str) incl 'A'."""
    case = json.load(open(f'{CASES}/{name}.case.json'))['sheets'][0]['cells']
    orc = json.load(open(f'{CASES}/{name}.oracle.json'))['sheets']['Calc']
    n = sum(1 for k in case if k.startswith('A') and k[1:].isdigit())
    rows = []
    for r in range(1, n + 1):
        d = {'row': r, 'A': float(case[f'A{r}'])}
        for col in 'BCDEFGH':
            c = case.get(f'{col}{r}')
            if c is None: continue
            if isinstance(c, (int, float)): d[col] = float(c)
            else:
                v = orc.get(f'{col}{r}')
                d[col] = float(v['v']) if v and v.get('t') == 'n' else ('!%s' % (v.get('v') if v else 'missing'))
        rows.append(d)
    return rows

def resid(x, y, E=None):
    """y relative to E*erfcx(x), in units of 2^-53 relative. E defaults to correctly rounded exp(-fl(x*x))."""
    X = mp.mpf(x); z = x * x
    erx = mp.erfc(X) * mp.exp(X * X)
    if E is None: E = float(mp.exp(-mp.mpf(z)))
    return float((mp.mpf(y) / (mp.mpf(E) * erx) - 1) / EPS)

def cr_exp(z):
    return float(mp.exp(-mp.mpf(z)))

def fl64(v):
    with mp.workprec(64):
        return +mp.mpf(v)

def x87mul(a, b):
    """fl53(fl64(a*b)) as a Python float (round-to-nearest-even at both steps)."""
    p = mp.mpf(a) * mp.mpf(b)          # exact at prec 140
    return float(fl64(p))              # float() rounds the 64-bit value to nearest double

def resid_z(x, y, z, E=None):
    """y relative to E*erfcx(x) where E = cr exp(-z) by default."""
    X = mp.mpf(x)
    erx = mp.erfc(X) * mp.exp(X * X)
    if E is None: E = float(mp.exp(-mp.mpf(z)))
    return float((mp.mpf(y) / (mp.mpf(E) * erx) - 1) / EPS)
