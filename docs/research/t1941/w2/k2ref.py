# K2 reference model (verbatim snippet) + end-to-end check on all clusters and full long runs
import sys, os, numpy as np
LD = np.longdouble
P = [float.fromhex(h) for h in ['0x1.5717994ef45d1p-51','0x1.5e5b7382714d8p-16','0x1.0ee2f11891de0p-9','0x1.572bf8684199dp-5','0x1.09fd1ad1d9003p-2','0x1.f0a8c053986d7p-2','-0x1.08305b47998b9p-6','-0x1.0ae34eb3cbad6p-1','-0x1.de852aee8c153p-3','-0x1.22260ed97843dp-6']]
Q = [float.fromhex(h) for h in ['0x1.03cadf2cb5f49p-18','0x1.52fc9914beb6ep-11','0x1.5303658ef4376p-6','0x1.bb43b464d397fp-3','0x1.c515177eb1641p-1','0x1.7b39fc9194438p+0','0x1.eddf2040d2bc2p-1','0x1.9b252b2fab347p-3','0x1.00000000000eap-7']]
def sd(v): return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)
def chain(c, x, K):
    """head (stages 1..K-1) in x87 80-bit registers; stored double Horner from stage K's product"""
    s = LD(c[0]) + 0 * x
    for i in range(1, len(c)):
        s = s * x + LD(c[i]) if i < K else sd(sd(s * x) + LD(c[i]))
    return s
def k2(x):
    x = np.asarray(x).astype(LD)
    return sd(sd(x - 2) * sd(chain(P, x, 4) / chain(Q, x, 3))).astype(np.float64)
if __name__ == '__main__':
    sys.path.insert(0, '/home/user/rustsheet/docs/research/t1941/review3'); os.chdir('/home/user/rustsheet')
    from spec import load
    m = []; bias = []
    for nn in range(1, 17):
        for h in 'ab':
            X0, Y0 = load('q2%02d%s' % (nn, h)); Yk = k2(X0)
            br = [0] + [i + 1 for i in range(len(X0) - 1) if np.nextafter(X0[i], np.inf) != X0[i + 1]] + [len(X0)]
            for a, b in zip(br, br[1:]): m.append(np.mean(Yk[a:b] == Y0[a:b])); bias.append(np.mean((Y0[a:b] - Yk[a:b]) / np.spacing(np.abs(Y0[a:b]))))
    lr = []
    for case in 'r701 r112 p401 r702 r610 r901 r703 r113 r704 r611 r705 v401a v404a v407a'.split():
        X0, Y0 = load(case); lr.append(np.mean(k2(X0) == Y0))
    print('clusters %.4f (n %d)  bias rms %.4f  | long-run cases (all points) %.4f : %s' % (np.mean(m), len(m), np.sqrt(np.mean(np.square(bias))), np.mean(lr), ' '.join('%.3f' % v for v in lr)))
