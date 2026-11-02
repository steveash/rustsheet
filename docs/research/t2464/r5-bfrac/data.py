"""T-3571 r5-bfrac: Excel BETA.DIST cdf/pdf pairs on runs of consecutive doubles, read from probe sidecars.

load(case, block, run_len) -> list of dicts x, a, b, cdf, pdf (Excel values). Cells alternate cdf (TRUE) / pdf
(FALSE); x is parsed exactly from the formula (literals cut to 15 digits as Excel does, numbers.md)."""
import json, re
from fractions import Fraction
from pathlib import Path
CASES = Path(__file__).resolve().parents[4] / 'eval/suites/probes-numeric/cases'

def lit(s):
    """Excel formula number literal -> double (15 significant digits, truncated)."""
    f = Fraction(s)
    if f == 0: return 0.0
    m = re.match(r'^(\d*)\.?(\d*)(?:[eE]([-+]?\d+))?$', s)
    digits = (m.group(1) + m.group(2)).lstrip('0')
    if len(digits) <= 15: return float(f)
    from decimal import Decimal, ROUND_DOWN, getcontext
    getcontext().prec = 60
    d = Decimal(s)
    q = d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_DOWN)
    return float(Fraction(q))

def ev(expr):
    """Exact value of an x argument: a literal or (m1*2^-e1+m2*2^-e2) pieces (exact sums, then one rounding)."""
    expr = expr.strip()
    if expr.startswith('('): expr = expr[1:-1]
    if '*2^' not in expr: return lit(expr)
    tot = Fraction(0)
    for term in re.findall(r'[-+]?\d+\*2\^-?\d+', expr):
        m, e = term.split('*2^'); tot += Fraction(int(m)) * Fraction(2) ** int(e)
    return float(tot)

def load(case, block=None, run_len=2048):
    c = json.load(open(CASES / f'{case}.case.json'))['sheets'][0]['cells']
    o = json.load(open(CASES / f'{case}.oracle.json'))['sheets']
    o = next(iter(o.values()))
    rows = {}
    for cell, f in c.items():
        m = re.match(r'^=BETA\.DIST\((.*),([^,]+),([^,]+),(TRUE|FALSE)\)$', f)
        if not m: continue
        i = int(cell[1:]) - 1
        if block is not None and i // run_len != block: continue
        x = ev(m.group(1)); a = lit(m.group(2)); b = lit(m.group(3))
        v = o.get(cell, {})
        if v.get('t') != 'n': continue
        d = rows.setdefault((x, a, b), dict(x=x, a=a, b=b, cell=cell))
        d['cdf' if m.group(4) == 'TRUE' else 'pdf'] = v['v']
    return [r for r in rows.values() if 'cdf' in r and 'pdf' in r]

if __name__ == '__main__':
    r = load('t2464-w2f-xruns-1', 7)
    print(len(r), r[0], r[-1])
