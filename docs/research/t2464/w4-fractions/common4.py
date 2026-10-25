"""w4-fractions shared helpers: data, CF runner, x87 arithmetic via numpy longdouble, scoring."""
import csv, math, subprocess, sys
from pathlib import Path
import numpy as np
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
LD = np.longdouble
assert np.finfo(LD).nmant == 63, "needs x87 80-bit long double"

def load(filt=None):
    rows = []
    for r in csv.DictReader(open(HERE / 'data.tsv'), delimiter='\t'):
        d = dict(src=r['src'], blk=r['blk'], x=float(r['x']), a=float(r['a']), b=float(r['b']), branch=r['branch'],
                 cdf=float(r['cdf']), pdf=float(r['pdf']), cdfx=r['cdfx'], pdfx=r['pdfx'])
        d['cdfcr'] = float(r['cdfx']); d['pdfcr'] = float(r['pdfx'])
        if filt is None or filt(d):
            rows.append(d)
    return rows

CF_NAMES = ['nr3e-16', 'nr2.2e-16', 'nr1e-15', 'nr1e-14', 'nr300', 'incbcf3', 'incbcf1', 'incbd3', 'lo',
            'bfrac1.5e-14', 'bfrac2.2e-16', 'bfracLy', 'back30', 'back60', 'back120', 'back240']

def parse_ld(s):
    # %La output: 0xh.hhhp+e  -> exact longdouble via fractions
    from fractions import Fraction
    neg = s.startswith('-'); s = s.lstrip('-')
    if s in ("inf", "nan"): return LD(float(s))
    if not s.startswith("0x"): return LD(int(s))
    m, e = s[2:].split('p')
    if '.' in m: ip, fp = m.split('.')
    else: ip, fp = m, ''
    mant = int(ip + fp, 16) if (ip + fp) else 0
    v = Fraction(mant, 16 ** len(fp)) * Fraction(2) ** int(e)
    # exact to longdouble: numerator/denominator are exact with <=64 bits mantissa
    r = LD(v.numerator) / LD(v.denominator) if v.denominator != 1 else LD(v.numerator)
    return -r if neg else r

def run_cf(rows, mode, exe='c/cf%d.bin'):
    inp = ''.join(f"{r['a'].hex()} {r['b'].hex()} {r['x'].hex()} {(1 - r['x']).hex()}\n" for r in rows)
    out = subprocess.run([str(HERE / (exe % mode))], input=inp, capture_output=True, text=True, check=True).stdout.split('\n')
    res = []
    for line in out[:len(rows)]:
        res.append([parse_ld(t) for t in line.split()])
    return res

def x(op, a, b):
    """stored x87: fl53(fl64(a op b)) for doubles/longdoubles"""
    A, B = LD(a), LD(b)
    r = A + B if op == '+' else A - B if op == '-' else A * B if op == '*' else A / B
    return float(r)

def ulpd(m, e):
    if m == e: return 0.0
    if not math.isfinite(m) or e == 0: return math.inf
    return abs(m - e) / math.ulp(e)

def w15(m, e):
    if m == e: return True
    if e == 0 or not math.isfinite(m): return False
    k = math.floor(math.log10(abs(e)))
    return abs(m - e) <= 0.5 * 10 ** (k - 14)

def score(ms, es):
    n = len(es); ex = sum(m == e for m, e in zip(ms, es)); w4 = sum(ulpd(m, e) <= 4 for m, e in zip(ms, es))
    w = sum(w15(m, e) for m, e in zip(ms, es))
    return n, ex, w4, w
