"""Binade-crossing events of a candidate's Horner intermediates on [0.7,1.47] (T-1941 W5 part 4, K3 plateau reader).
My naming: stage j of chain C is P_j = fl(S_{j+1}*x), S_j = fl(P_j + c_j). Prints events with data counts in
0.002-wide windows either side."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, os.path.join(HERE, '..'))
from sc import candidate, load
def events(N, D, lo=0.7, hi=1.47, n=800001):
    x = np.linspace(lo, hi, n); out = []
    for ch, C in (('N', N), ('D', D)):
        s = np.full_like(x, C[-1])
        for j in range(len(C)-2, -1, -1):
            p = s*x; s = p+C[j]
            for nm, v in (('P', p), ('S', s)):
                e = np.floor(np.log2(np.abs(v)+1e-300)); ch_ = np.where(np.diff(e) != 0)[0]
                for i in ch_: out.append((x[i], '%s.%s%d' % (ch, nm, j), int(e[i]), int(e[i+1])))
    A = np.polyval(N[::-1], x)/np.polyval(D[::-1], x); r = (x-1)*A
    for nm, v in (('A', A), ('res', r)):
        e = np.floor(np.log2(np.abs(v)+1e-300)); ch_ = np.where(np.diff(e) != 0)[0]
        for i in ch_:
            if abs(x[i]-1) > 0.003: out.append((x[i], nm, int(e[i]), int(e[i+1])))
    return sorted(out)
if __name__ == '__main__':
    N = candidate('N3'); D = candidate('D3'); xs, b, c = load(); W = 0.002
    for xe, nm, e0, e1 in events(N, D):
        nl = int(np.sum((xs >= xe-W) & (xs < xe))); nr = int(np.sum((xs > xe) & (xs <= xe+W)))
        print('%.5f %-7s 2^%d->2^%d  data L %6d R %6d' % (xe, nm, e0, e1, nl, nr))
