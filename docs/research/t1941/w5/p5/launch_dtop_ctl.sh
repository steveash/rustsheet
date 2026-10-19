#!/bin/sh
cd "$(dirname "$0")"
export ROWS=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows3.txt RL=16384 CHAIN=D J=6 TRUTHFILE=e2e_truth.txt CMAX=50 CSTEP=0.5 DMAX=2 DSTEP=0.5 NB=21 BMAX=1e5 H=4 KEEP=10
setsid nohup python3 scan3.py control > scan3_D6_ctl.log 2>&1 < /dev/null &
