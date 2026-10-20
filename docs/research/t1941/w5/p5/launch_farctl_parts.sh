#!/bin/sh
# Range control (resumable, 4 parallel parts): wide N.S7 box (b +-1e5, c +-2000), planted truth at TSIZE 1e7.
# Rerun this script after a restart: each part resumes from its checkpoint. Merge: MERGE=1 CKPTS=... (see README).
cd "$(dirname "$0")"
S=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad
export ROWS=$S/rows3.txt RL=16384 CMAX=2000 CSTEP=0.5 NB=21 BMAX=1e5 H=4 KEEP=10 TSIZE=1e7
for i in 0 1 2 3; do
  CPART=$i/4 CKPT=$S/ck_farctl_$i.txt setsid nohup python3 scan2.py control > scan_farctl_NS7_p$i.log 2>&1 < /dev/null &
done
