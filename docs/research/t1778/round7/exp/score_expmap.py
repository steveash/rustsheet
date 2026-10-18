"""Score kmyst-erfc-expmap1/2 once the oracle has filled them.
usage: python3 score_expmap.py [CASES_DIR]   (default: this dir's probe/ for a dry run with fake oracles)
Outputs: probe_expdata.txt (a E tag), model match counts, cluster test (is one integer F2XM1 error k shared by
all 20 hits of a cluster/exponent?), repeat reproducibility, split-discriminator verdict."""
import json, sys, pickle, collections, mpmath as mp
sys.path.insert(0, '/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/inv4/exp-exact')
from exp_ref import excel_exp, _r64
CD = sys.argv[1] if len(sys.argv) > 1 else '/tmp/claude-0/-home-user-rustsheet/a0a8642a-3715-514e-9c19-b7216ee09dd7/scratchpad/inv4/exp-exact/probe'
mp.mp.prec = 256
L2E64 = _r64(1 / mp.log(2))
def load(name):
    c = json.load(open(f'{CD}/{name}.case.json'))['sheets'][0]['cells']
    o = json.load(open(f'{CD}/{name}.oracle.json'))['sheets']['Calc']
    n = sum(1 for k in c if k[0] == 'A')
    return [(c[f'A{i}'], o[f'B{i}']['v'] if o.get(f'B{i}', {}).get('t') == 'n' else None) for i in range(1, n + 1)]
def kset(a, E, K=3, split='floor'):
    t = _r64(mp.mpf(a) * L2E64); n = mp.floor(t) if split == 'floor' else mp.nint(t)
    f = _r64(t - n); us = mp.power(2, f) - 1; u0 = _r64(us)
    m, e = mp.frexp(u0); ul = mp.mpf(2) ** (e - 64)
    return tuple(float(mp.ldexp(_r64(1 + u0 + k * ul), int(n))) == E for k in range(-K, K + 1))
m1 = load('kmyst-erfc-expmap1'); m2 = load('kmyst-erfc-expmap2')
with open('probe_expdata.txt', 'w') as fo:
    for tag, rows in (('expmap1', m1), ('expmap2', m2)):
        for a, E in rows:
            if E is not None: fo.write(f'{a.hex()} {E.hex()} {tag}\n')
allr = [(a, E) for a, E in m1 + m2 if E]
ok = sum(excel_exp(a) == E for a, E in allr); print(f'CR-u floor model: {ok}/{len(allr)} (nonzero cells)')
# cluster test
K = 3; incons = 0; ncl = 0; hist = collections.Counter()
for c in range(300):
    for j in range(5):
        rows = m1[100 * c + 20 * j: 100 * c + 20 * j + 20]
        S = [kset(a, E, K) for a, E in rows if E]
        inter = [all(s[i] for s in S) for i in range(2 * K + 1)]
        ncl += 1
        if not any(inter): incons += 1
        else: hist[tuple(i - K for i in range(2 * K + 1) if inter[i])] += 1
print(f'clusters with no common k: {incons}/{ncl}; common-k sets (top): {hist.most_common(8)}')
ns = 30000; nd = 2000
sd = m2[ns:ns + nd]
t = sum(excel_exp(a) == E for a, E in sd if E); print(f'split discriminators: floor-rounded-f model matches {t}/{len(sd)}')
rp = m2[ns + nd:]; h = len(rp) // 2
print('repeat pairs identical:', sum(rp[i][1] == rp[i + h][1] for i in range(h)), '/', h)
