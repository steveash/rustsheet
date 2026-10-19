#!/bin/sh
cd "$(dirname "$0")"
export ROWS=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows3.txt RL=16384
cur=e2e_dtop.txt
for j in 5 4 3 2 1; do
  out=$(BASE=$cur TRUTHFILE=e2e_truth.txt KRANGE=2e6 python3 fscan.py control D $j); echo "$out"
  k=$(echo "$out" | head -1 | sed 's/.*top: k=\([+-][0-9]*\).*/\1/')
  python3 -c "
import numpy as np, sys
sys.path.insert(0, '.')
from hypolib import ld
N, D = ld('$cur'); D[$j] += ($k)*np.spacing(abs(D[$j]))
open('e2e_e$j.txt', 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')"
  cur=e2e_e$j.txt
done
