#!/bin/sh
# Real data, D top step 1: cubic-less (DMAX 0) D.S6 scan on rows4 (incl. V3), c +-200
cd "$(dirname "$0")"
export ROWS=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt RL=16384 CHAIN=D J=6 CMAX=200 CSTEP=0.5 DMAX=0 NB=21 BMAX=1e5 H=4 KEEP=10
setsid nohup python3 scan3.py real > scan3_D6_real.log 2>&1 < /dev/null &
