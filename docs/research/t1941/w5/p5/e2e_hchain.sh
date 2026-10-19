#!/bin/sh
# Full control chain, D side from scratch: N peeled (e2e_f1) + D7..D9 from dtop3 (e2e_g3); peel D6..D1, then N0.
cd "$(dirname "$0")"
export ROWS=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows3.txt RL=16384 TRUTHFILE=e2e_truth.txt
cur=e2e_h7.txt
for j in 6 5 4 3 2 1; do ./peelstep.sh control D $j $cur e2e_h$j.txt; cur=e2e_h$j.txt; done
python3 n0scan.py control $cur e2e_hfinal.txt
