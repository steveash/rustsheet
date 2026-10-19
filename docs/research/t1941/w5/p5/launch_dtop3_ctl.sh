#!/bin/sh
# D top on the control, J0=6 objective: (D7, D8, D9) from e2e_g2 (scan3 D7/D9 + topgrid2 step-1 D8)
cd "$(dirname "$0")"
export ROWS=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows3.txt RL=16384 TRUTHFILE=e2e_truth.txt
setsid nohup python3 dtop3.py control e2e_g2.txt e2e_g3.txt > e2e_dtop3.log 2>&1 < /dev/null &
