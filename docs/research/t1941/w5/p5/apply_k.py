"""Apply a move of k ulps to c_j and refit the lower/other free coefficients to a target function (T-1941 W5 part 5).
usage: apply_k.py IN OUT CHAIN J K TARGET [EXCL]"""
import numpy as np, sys, os, subprocess
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
fin, fout, CH, J, K, tgt = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5]), sys.argv[6]
excl = sys.argv[7] if len(sys.argv) > 7 else ''
N, D = ld(fin); C = N if CH == 'N' else D; C[J] += K*np.spacing(abs(C[J]))
tmp = fout+'.tmp'; open(tmp, 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')
# lower coefficients of this chain below J plus the other chain; keep the already-solved upper part fixed
subprocess.run([sys.executable, os.path.join(HERE, 'refit_fn.py'), tmp, tgt, fout, CH, str(J), excl], check=True)
os.remove(tmp)
