import sys, os, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from k3lib import *
x, b, _ = rows(sub=int(os.environ.get('SUB', 300000)), seed=1)
p, q = ldset('docs/research/t1941/w5/p4/ss_fc.txt')
F = int(os.environ.get('F', 1)); KM = int(os.environ.get('KM', 20000)); ST = int(os.environ.get('ST', 1))
print('base %.4f' % score(x, b, p, q), flush=True)
for tag in os.environ.get('STAGES', 'N3 D5 N7 D6 N2 D1 N8 D7 N1 D8').split():
    side, L = tag[0], int(tag[1:]); c = list(p if side == 'N' else q); u = float(np.spacing(abs(c[L])))
    ks = np.arange(-KM, KM + 1, ST); r = np.empty(len(ks))
    for j, k in enumerate(ks):
        cc = float(k * u); cm = list(c); cm[L - 1] = float(np.float64(c[L - 1]) - cc); o = (L, F, cc, 0, 0)
        r[j] = score(x, b, cm, q, o) if side == 'N' else score(x, b, p, cm, NOPT, o)
    o = np.argsort(r)[::-1]
    print('%s form %d: k=0 %.4f median %.4f p99.9 %.4f | top %s' % (tag, F, r[ks == 0][0], np.median(r), np.percentile(r, 99.9), ' '.join('%d:%.4f' % (ks[i], r[i]) for i in o[:8])), flush=True)
