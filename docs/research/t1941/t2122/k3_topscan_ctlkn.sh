#!/bin/sh
# T-2122 run 5 (HANDOFF2 17.7): unstored-top control. As k3_topscan_ctl.sh, but B has the N ops j >= KN (7) in 80-bit. Same box, rows and settings; base = the control's
# Newton end point c7 (k3_run5_ctl_c7.txt), truth = k3_run5_ctl_truth.txt (B synthesised from it at the real x).
# usage (from docs/research/t1941): k3_topscan_ctl.sh PY ROWS
PY=$1; ROWS=$2; T=$(pwd)/t2122
cd w5/p5 || exit 1
export ROWS BASE=$T/k3_run5_ctl_c7.txt TRUTHFILE=$T/k3_run5_ctl_truth.txt RL=16384 CMAX=1800 CSTEP=0.5 NB=21 BMAX=5e3 H=4 KEEP=10 CORDER=center KN=${KN:-7}
for i in 0 1 2 3; do
  CPART=$i/4 CKPT=$T/ck_run5_ctlkn_$i.txt setsid nohup "$PY" scan2.py control > "$T/k3_topscan_ctlkn_p$i.log" 2>&1 < /dev/null &
done
