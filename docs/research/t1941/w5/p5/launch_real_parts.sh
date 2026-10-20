#!/bin/sh
# Real wide N.S7 scan (resumable, 4 parts) on rows6: clean K3 set (B = GAMMALN only; q501 excluded), incl. k3t.
# Rerun after a restart to resume. Merge: MERGE=1 CKPTS=ck_real_0..3 (README).
cd "$(dirname "$0")"
S=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad
export ROWS=$S/rows6.txt RL=16384 CMAX=2000 CSTEP=0.5 NB=21 BMAX=1e5 H=4 KEEP=10
for i in 0 1 2 3; do
  CPART=$i/4 CKPT=$S/ck_real_$i.txt setsid nohup python3 scan2.py real > scan_real_NS7_p$i.log 2>&1 < /dev/null &
done
