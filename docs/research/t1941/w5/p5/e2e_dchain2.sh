#!/bin/sh
cd "$(dirname "$0")"
export ROWS=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows3.txt RL=16384 TRUTHFILE=e2e_truth.txt
cur=e2e_e5.txt
for j in 4 3 2 1; do ./peelstep.sh control D $j $cur e2e_f$j.txt; cur=e2e_f$j.txt; done
