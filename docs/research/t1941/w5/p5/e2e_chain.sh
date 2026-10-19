#!/bin/sh
# End-to-end control, N chain from N6 down, no function refit: fscan each coefficient, apply the top hit.
cd "$(dirname "$0")"
cur=${START:-e2e_top.txt}
for j in 6 5 4 3 2 1 0; do
  out=$(BASE=$cur TRUTHFILE=e2e_truth.txt KRANGE=${KRANGE:-2e6} python3 fscan.py control N $j)
  echo "$out"
  k=$(echo "$out" | head -1 | sed 's/.*top: k=\([+-][0-9]*\).*/\1/')
  python3 -c "
import numpy as np, sys
sys.path.insert(0, '.')
from hypolib import ld
N, D = ld('$cur'); N[$j] += ($k)*np.spacing(abs(N[$j]))
open('e2e_d$j.txt', 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')"
  cur=e2e_d$j.txt
done
