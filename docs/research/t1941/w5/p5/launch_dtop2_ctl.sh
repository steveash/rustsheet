#!/bin/sh
# D top on the control: scan3 cubic -> dtop.py (d = 0) -> topgrid2 on CHAIN=D (D7..D9), wide grid
cd "$(dirname "$0")"
export ROWS=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows3.txt RL=16384 TRUTHFILE=e2e_truth.txt CHAIN=D R8=512 S8=4 R9=262144 S9=2048
setsid nohup python3 topgrid2.py control e2e_g0.txt e2e_g1.txt > e2e_dtop2.log 2>&1 < /dev/null &
