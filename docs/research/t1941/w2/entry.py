exec(open('hvar.py').read())
from multiprocessing import Pool
NP = int(os.environ.get('NP', 512)); D4 = [(X[:NP], Y[:NP]) for X, Y in DATA]
U = 2.0 ** -54
def chE(c, x, K, cc, form, cb=0.0):
    c = list(c); s = LD(c[0]) + 0 * x; T = LD(cc) / x
    if form == 'J0': s = s + LD(cc) / x ** (K + 1)
    for i in range(1, len(c)):
        ci = LD(c[i]) - (LD(cc) + LD(cb) if i == K + 1 else 0)
        if i == K - 1 and form == 'E1': s = sd(sd(s * x) + ci + LD(cc) / (x * x)); continue
        if i == K and form != 'J0':
            if form == 'E1': s = sd(sd(s * x) + ci)
            elif form == 'E2': s = sd(sd((s + LD(cc) / (x * x)) * x) + ci)
            elif form == 'E3': s = sd(sd(s * x + T) + ci)
            elif form == 'E4': s = sd(sd(sd(s * x) + T) + ci)
            elif form == 'E5': s = sd(sd(s * x) + ci + T)
            elif form == 'Eb': s = sd(s * x + T + ci)
            elif form == 'Ec': s = sd(sd(s * x + T) + sd(ci + LD(cb) / x))
            continue
        s = sd(sd(s * x) + ci)
    return s
def yv(X, fN, cN, fD, cD): return sd(sd(X - LD(2)) * sd(chE(P0, X, 4, cN, fN) / chE(Q0, X, 3, cD, fD))).astype(np.float64)
def sc(a):
    fN, kN, fD, kD = a[:4]; data = a[4] if len(a) > 4 else D4
    return float(np.mean([np.mean(yv(X, fN, kN * U, fD, kD * U) == Y) for X, Y in data]))
FORMS = ['E5', 'E1', 'E2', 'E3', 'E4', 'Eb', 'J0']
if __name__ == '__main__':
    with Pool(4) as pool:
        for side in ('N', 'D'):
            for f in FORMS:
                ks = np.arange(-3170, -3085) if side == 'N' else np.arange(510, 595)
                cfg = [(f, k, 'E5', 552) if side == 'N' else ('E5', -3128, f, k) for k in ks]
                r = np.array(pool.map(sc, cfg)); o = np.argsort(r)[::-1]; kb = ks[o[0]]
                tr = cfg[o[0]]; PL = [(X, yv(X, tr[0], tr[1] * U, tr[2], tr[3] * U)) for X, Y in D4[:4]]
                ctl = max(sc(((('E5', k, 'E5', 552) if side == 'N' else ('E5', -3128, 'E5', k)) + (PL,))) for k in ks[::4])
                print('%s %-3s: best c=%d %.4f (2nd %d:%.4f, median %.4f) | plant: self %.3f, E5-best-c %.3f' % (side, f, kb, r[o[0]], ks[o[1]], r[o[1]], np.median(r), sc(tr + (PL,)), ctl), flush=True)
