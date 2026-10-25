"""T-2144: fit LN's FYL2X near-ties with a k-bit internal product (truncated `tz` or rounded `rn`)
before RN64 and RN53, against the points where all three T-2143 fills agree and the T-2123 fixture.
k=0 is the shipped model. Run: python3 docs/research/t1941/ln_guard_bits.py (stdlib only)."""
import json, struct, sys
from decimal import Decimal, getcontext
from fractions import Fraction
getcontext().prec = 70
R = str(__import__('pathlib').Path(__file__).resolve().parents[3]) + '/'
LN2 = Decimal(2).ln()

def frac_round(v, bits, mode):
    """Round positive/negative Fraction v to `bits` significant bits; mode 'rn' (ties even) or 'tz'."""
    if v == 0: return v
    s = -1 if v < 0 else 1; a = abs(v)
    e = a.numerator.bit_length() - a.denominator.bit_length()
    while Fraction(2)**e > a: e -= 1
    while Fraction(2)**(e+1) <= a: e += 1
    scale = Fraction(2)**(bits-1-e)
    m = a*scale; q = m.numerator // m.denominator; r = m - q
    if mode == 'rn':
        if r > Fraction(1,2) or (r == Fraction(1,2) and q & 1): q += 1
    return s*Fraction(q)/scale

L2 = frac_round(Fraction(LN2), 64, 'rn')
def log2(x):
    return Fraction(Decimal(x).ln() / LN2)

def model(x, k, mode):
    p = L2*log2(x)
    if k: p = frac_round(p, k, mode)
    return float(frac_round(frac_round(p, 64, 'rn'), 53, 'rn'))

def fbits(h): return struct.unpack('>d', bytes.fromhex(h))[0]
# T-2143 fills
c = json.load(open(R+'eval/suites/probes-numeric/cases/t2143-ln-near-ties-fill1.case.json'))['sheets'][0]['cells']
fills = [json.load(open(R+f'eval/suites/probes-numeric/cases/t2143-ln-near-ties-fill{i}.oracle.json'))['sheets']['Calc'] for i in (1,2,3)]
stable = []; unstable = 0
for k, v in c.items():
    if not k.startswith('A'): continue
    b = 'B'+k[1:]
    vals = [f.get(b, {}).get('v') for f in fills]
    if None in vals: continue
    if len(set(vals)) == 1: stable.append((v, vals[0]))
    else: unstable += 1
fix = []
for line in open(R+'crates/rustsheet-engine/src/test/fixtures/ln_kernel.fixture'):
    if line.startswith('#') or not line.strip(): continue
    x, ex, _ = line.split(); fix.append((fbits(x), fbits(ex)))
print('stable', len(stable), 'unstable', unstable, 'fixture', len(fix))
cands = [(0, 'rn')] + [(k, m) for k in (66, 67, 68, 69, 70, 72) for m in ('tz', 'rn')]
for k, m in cands:
    s = sum(model(x, k, m) == y for x, y in stable)
    f = sum(model(x, k, m) == y for x, y in fix)
    print(f'k={k} {m}: stable {s}/{len(stable)} fixture {f}/{len(fix)}', flush=True)
print('--- breakdown')
base = [model(x, 0, 'rn') == y for x, y in fix]
for k, m in [(0,'rn'),(66,'rn'),(67,'tz'),(67,'rn'),(68,'tz')]:
    r = [model(x, k, m) == y for x, y in fix]
    a = sum(1 for i in range(len(fix)) if base[i] and r[i]); b = sum(1 for i in range(len(fix)) if not base[i] and r[i])
    print(f'k={k} {m}: keeps {a}/{sum(base)} model-agreeing fixture points, {b}/{len(fix)-sum(base)} of the near-tie misses')
