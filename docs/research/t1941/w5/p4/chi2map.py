import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
sys.argv = sys.argv[:2] + ['none']
exec(open(os.path.join(HERE, 'plateau.py')).read().split("if __name__")[0])
def ld(f):
    l = open(f).read().split('\n'); return np.array([float.fromhex(v) for v in l[0].split()]), np.array([float.fromhex(v) for v in l[1].split()])
tmu, tse = runmean(b)
for tag, (N, D) in [('c5', (candidate('N3'), candidate('D3'))), ('bits+restored', ld(os.path.join(HERE, 'k3_bits_restored.txt'))), ('+coord search', ld(os.path.join(HERE, 'plateau_real.txt')))]:
    c2, r = chi2(tmu, tse, N, D)
    edges = np.arange(0.70, 1.48, 0.04); line = []
    for lo in edges:
        m_ = (xr >= lo) & (xr < lo+0.04); line.append('%.2f:%d' % (lo, int((r[m_]**2).sum())))
    print('%-14s chi2 %.0f |' % (tag, c2), ' '.join(line))
