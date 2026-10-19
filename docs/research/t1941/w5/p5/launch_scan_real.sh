#!/bin/sh
# detached real-data N.S7 wide scan (survives the tool's background time limit)
cd "$(dirname "$0")"
export ROWS=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows3.txt RL=16384 CMAX=2000 CSTEP=0.5 NB=21 BMAX=1e5 H=4 KEEP=10
setsid nohup python3 scan2.py real > scan_real_NS7_wide.log 2>&1 < /dev/null &
