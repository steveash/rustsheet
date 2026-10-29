#!/bin/sh
# T-2122 run 7 (HANDOFF2 19): the 18.3/18.4 J=6 ridge scans with the base's own bias subtracted (scan3.py BIASFILE):
# the residual the base set leaves under an 80-bit S_7/S_8 (KN = 7) is removed from the data before the scan, which
# opens the |b| <= 30 region that the base's sidelobes hid in 18.4. Same grids and boxes as k3_j6scan.sh.
# usage (from docs/research/t1941): k3_j6bias.sh PY ROWS real|control OUTDIR
PY=$1; ROWS=$2; MODE=$3; OUT=$4; T=$(pwd)/t2122
cd w5/p5 || exit 1
if [ "$MODE" = control ]; then
  export BASE=$T/k3_run5_ctl_c7.txt TRUTHFILE=$T/k3_run5_ctl_truth.txt KN=7 CMAX=250 DMAX=628 RIDGE=0.380,75
  export BIASFILE=$T/k3_run5_ctl_c7.txt
else
  export BASE=$T/k3_run5_C.txt CMAX=196 DMAX=412 RIDGE=0.440,74
  export BIASFILE=$T/k3_run5_C.txt
fi
export BIASKN=7 ROWS J=6 RL=16384 NB=21 BMAX=5e3 H=4 KEEP=1 NSAVE=100000 CSTEP=2 DSTEP=4 TMPDIR_W5=$OUT
for i in 0 1 2 3; do
  CPART=$i/4 setsid nohup "$PY" scan3.py "$MODE" > "$T/k3_j6bias_${MODE}_p$i.log" 2>&1 < /dev/null &
done
