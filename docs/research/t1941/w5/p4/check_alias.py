import os, sys, numpy as np
sys.argv = ['plateau.py', 'none']
exec(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'plateau.py')).read().split("if __name__")[0])
def mv(spec):
    N = N5.copy(); D = D5.copy()
    for (ch, k), j in spec.items():
        C = N if ch == 'N' else D; C[k] += j*np.spacing(abs(C[k]))
    return N, D
Nt, Dt = mv({('N', 0): 37, ('D', 1): -11, ('D', 2): 3})
tmu, tse = runmean(res(x, Nt, Dt))
for nm, spec in [('c5', {}), ('truth', {('N', 0): 37, ('D', 1): -11, ('D', 2): 3}), ('found', {('N', 0): 948, ('D', 1): -15, ('D', 2): -6}),
                 ('truth but N0=948', {('N', 0): 948, ('D', 1): -11, ('D', 2): 3}), ('truth but D2=-6', {('N', 0): 37, ('D', 1): -11, ('D', 2): -6})]:
    print('%-18s chi2 %.1f' % (nm, chi2(tmu, tse, *mv(spec))[0]))
print('runs', R, 'points', len(x))
