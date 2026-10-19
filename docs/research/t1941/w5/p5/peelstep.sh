#!/bin/sh
# One peel step: FFT shortlist (fscan.py), exact re-score of the top candidates and +-2R neighbours (peel2.py KLIST),
# apply the winner. usage: peelstep.sh MODE CHAIN J IN OUT   env: ROWS, RL, TRUTHFILE, KRANGE
cd "$(dirname "$0")"
MODE=$1; CH=$2; J=$3; IN=$4; OUT=$5
S=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad
BASE=$IN KRANGE=${KRANGE:-2e6} TOP=${TOP:-20} python3 fscan.py $MODE $CH $J | head -2
KL=$(python3 -c "
import numpy as np
t = np.load('$S/fscan_top_$CH$J.npy'); R = 8
ks = sorted(set(int(k)+d for k in t[:20, 0] for d in range(-2*R, 2*R+1)))
print(','.join(str(k) for k in ks))")
res=$(BASE=$IN KLIST=$KL python3 peel2.py $MODE $CH $J)
echo "$res" | sed 's/^/   exact: /'
k=$(echo "$res" | head -1 | sed 's/.*top: k=\([+-][0-9]*\).*/\1/')
python3 -c "
import numpy as np, sys
sys.path.insert(0, '.')
from hypolib import ld
N, D = ld('$IN'); C = N if '$CH' == 'N' else D; C[$J] += ($k)*np.spacing(abs(C[$J]))
open('$OUT', 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')"
echo "   applied $CH$J k=$k -> $OUT"
