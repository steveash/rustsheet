import pickle, sys, os, numpy as np
from multiprocessing import Pool
sys.path.insert(0, '/home/user/rustsheet/docs/research/t1941/review3'); os.chdir('/home/user/rustsheet')
from spec import load
LD = np.longdouble
def sd(v): return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)
PP, QQ = pickle.load(open(os.environ['START'], 'rb'))
LR = []
for case in 'r701 r112 p401 r702 r610 r901 r703 r113 r704 r611 r705 v401a v404a v407a'.split():
    X0, Y0 = load(case); br = [0] + [i + 1 for i in range(len(X0) - 1) if np.nextafter(X0[i], np.inf) != X0[i + 1]] + [len(X0)]
    a, b = max(zip(br, br[1:]), key=lambda t: t[1] - t[0]); LR.append((X0[a:b][:1024].astype(LD), Y0[a:b][:1024]))
MODES = ['S', 'U', 'Pst', 'Ast']
def st(s, x, c, m):
    if m == 'S': return sd(sd(s * x) + c)
    if m == 'U': return s * x + c
    if m == 'Pst': return sd(s * x) + c
    if m == 'Ast': return sd(s * x + c)
def ch(c, x, M):
    s = LD(c[0]) + 0 * x
    for i in range(1, len(c)): s = st(s, x, LD(c[i]), M.get(i, 'S'))
    return s
def sc(cfg):
    MN, MD = cfg; return float(np.mean([np.mean(sd(sd(X - LD(2)) * sd(ch(PP, X, MN) / ch(QQ, X, MD))).astype(np.float64) == Y) for X, Y in LR]))
SLOTS = [('N', i) for i in range(1, 10)] + [('D', i) for i in range(1, 9)]
cur = {'N': {}, 'D': {}}
with Pool(4) as pool:
    best = sc((cur['N'], cur['D'])); print('start %.4f' % best, flush=True)
    for rnd in range(3):
        moved = False
        for side, i in SLOTS:
            cands = []
            for m in MODES:
                c = {'N': dict(cur['N']), 'D': dict(cur['D'])}; c[side][i] = m; cands.append(c)
            res = pool.map(sc, [(c['N'], c['D']) for c in cands]); j = int(np.argmax(res))
            if rnd == 0: print('  %s%d: %s' % (side, i, ' '.join('%s:%.4f' % (m, r) for m, r in zip(MODES, res))), flush=True)
            if res[j] > best + 5e-4: best = res[j]; cur = cands[j]; moved = True; print('   -> %s %.4f' % (cur, best), flush=True)
        print('round', rnd, '%.4f %s' % (best, cur), flush=True)
        if not moved: break
