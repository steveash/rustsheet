import sys, pickle
from dataclasses import replace
sys.path.insert(0, '.')
from vscore import *
import bratio_excel as bx
rows, vals, br = pickle.load(open('run1.pkl', 'rb'))
xr = [(i, r) for i, r in enumerate(rows) if r['set'] == 'x29' and 0.29 <= r['x'] < 0.3]
v29, _ = model_values([r for _, r in xr], replace(bx.BEST, x29=0.29))
d = [(vals[i], b, r['excel'], r) for (i, r), b in zip(xr, v29) if vals[i] != b]
print('cells in [0.29,0.3)', len(xr), 'dispatch differs', len(d))
print('excel==0.3 model', sum(a == e for a, b, e, r in d), 'excel==0.29 model', sum(b == e for a, b, e, r in d))
h = [t for t in d if sha_half(repr(key(t[3])))]
print('sha1 half: 0.3', sum(a == e for a, b, e, r in h), '0.29', sum(b == e for a, b, e, r in h), 'of', len(h))
print('w15 0.3', sum(w15(a, e) for a, b, e, r in d), 'w15 0.29', sum(w15(b, e) for a, b, e, r in d))
print('mean |ulp| 0.3', sum(ulpd(a, e) for a, b, e, r in d) / len(d), '0.29', sum(ulpd(b, e) for a, b, e, r in d) / len(d))
# ix L140 out-of-sample claim
ixl = [(i, r) for i, r in enumerate(rows) if r['set'] == 'ix' and br[i] == 'L140 bup+bpser']
cr = pickle.loads(CRF.read_bytes())
print('ix L140 bup+bpser n', len(ixl), 'BEST', tally((vals[i], r['excel']) for i, r in ixl), 'CR(kernel args)', tally((cr[key(r)], r['excel']) for i, r in ixl))
