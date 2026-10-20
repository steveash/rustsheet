"""Control for the op-repair fingerprint (T-1941 W5 part 5): synthetic B from a truth set (e2e_truth, a
function-preserving move of ss_fc) under a given structure (TRUTHOPS unstored), then the oprep screen with ss_fc
constants. Positive control TRUTHOPS=D.S3 should flag D.S3; negative control TRUTHOPS='' should not.
usage: oprep_ctl.py TRUTHOPS   env: NSUB, COMBOS"""
import numpy as np, sys, os, subprocess
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from meter import L, st
ops = [o for o in sys.argv[1].split(',') if o]
z = np.load('/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz')
i = np.random.default_rng(1).choice(len(z['x']), int(os.environ.get('NSUB', 400000)), replace=False); x = z['x'][i]
N, D = ld(os.path.join(HERE, 'e2e_truth.txt')); xl = L(x)
def chain(C, skip):
    s = L(C[-1])
    for j in range(len(C)-2, -1, -1):
        p = s*xl; p = p if ('P', j) in skip else L(st(p))
        s = p+L(C[j]); s = s if ('S', j) in skip else L(st(s))
    return s
sk = {'N': set(), 'D': set()}
for o in ops: ch, k = o.split('.'); sk[ch].add((k[0], int(k[1:])))
b = st(L(st(xl-L(1.0)))*L(st(chain(N, sk['N'])/chain(D, sk['D']))))
out = '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/oprep_ctl.npz'
np.savez(out, x=x, b=b)
env = dict(os.environ); env['NSUB'] = '0'
subprocess.run([sys.executable, os.path.join(HERE, 'oprep.py'), os.path.join(HERE, '..', 'p4', 'ss_fc.txt'), out], env=env)
