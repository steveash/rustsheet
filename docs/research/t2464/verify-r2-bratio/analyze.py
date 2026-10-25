import sys, pickle
from collections import defaultdict, Counter
sys.path.insert(0, '.')
from vscore import *

rows, vals, br = pickle.load(open('run1.pkl', 'rb'))
cr = pickle.loads(CRF.read_bytes())
for r, v, b in zip(rows, vals, br):
    r['m'] = v; r['br'] = b; r['cr'] = cr[key(r)]


def line(name, rs):
    n, e, u, w = tally((r['m'], r['excel']) for r in rs)
    _, ce, cu, cw = tally((r['cr'], r['excel']) for r in rs)
    print(f"| {name} | {n} | {e}/{u}/{w} | {ce}/{cu}/{cw} |")


print("| subset | n | BEST ex/u4/w15 | CR ex/u4/w15 |\n|---|---|---|---|")
main = [r for r in rows if not r['set'].startswith('fresh')]
line('all 5 sets (own parse)', main)
for s in ['be', 'w2f', 'w4p1', 'ix', 'x29']:
    line(s, [r for r in main if r['set'] == s])
line('without x29', [r for r in main if r['set'] != 'x29'])
line('ix only, no closed', [r for r in main if r['set'] == 'ix' and not r['br'].startswith('closed')])
# dedupe: one row per (key), drop keys whose excel values disagree
g = defaultdict(list)
for r in main:
    g[key(r)].append(r)
dd = [rs[0] for rs in g.values() if len({x['excel'] for x in rs}) == 1]
conf = sum(1 for rs in g.values() if len({x['excel'] for x in rs}) > 1)
print('dup keys', sum(len(v) - 1 for v in g.values()), 'conflicting keys', conf)
line('deduped', dd)
# own hold-outs
line('sha1 half (cell key)', [r for r in dd if sha_half(repr(key(r)))])
line('sha1 other half', [r for r in dd if not sha_half(repr(key(r)))])
shp = lambda r: repr((r['set'], round(r['a'], 6), round(r['b'], 6)))
line('shape-grouped half A', [r for r in dd if sha_half(shp(r))])
line('shape-grouped half B', [r for r in dd if not sha_half(shp(r))])
print('n shapes', len({shp(r) for r in dd}))
# fresh
fr = [r for r in rows if r['set'].startswith('fresh')]
line('FRESH all (r2-bpser-small probes, after r2-bratio finished)', fr)
for s in sorted({r['set'] for r in fr}):
    line(s, [r for r in fr if r['set'] == s])
print('\nfresh by branch')
for b in sorted({r['br'] for r in fr}):
    line(b, [r for r in fr if r['br'] == b])
print('\nmain by branch')
for b in sorted({r['br'] for r in main}):
    line(b, [r for r in main if r['br'] == b])
