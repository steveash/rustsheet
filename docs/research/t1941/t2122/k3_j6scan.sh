#!/bin/sh
# T-2122 run 6 (HANDOFF2 18): J=6 scan (N.S6 = N6 + x S7, cubic dS) from C, on real data or the KN=7 control, over the
# ladder's 3-sigma c-d ridge from k3_topbox6.py at the coarse grid the control's peak width allows (c step 2, d step 4).
# usage (from docs/research/t1941): k3_j6scan.sh PY ROWS real|control OUTDIR
PY=$1; ROWS=$2; MODE=$3; OUT=$4; T=$(pwd)/t2122
cd w5/p5 || exit 1
if [ "$MODE" = control ]; then
  # control box from c7: sigma_c 83.3, sigma_d 209, corr cd 0.954 -> slope 0.380, half 3*24.8
  export BASE=$T/k3_run5_ctl_c7.txt TRUTHFILE=$T/k3_run5_ctl_truth.txt KN=7 CMAX=250 DMAX=628 RIDGE=0.380,75
else
  # real box from C: sigma_c 65.1, sigma_d 137, corr cd 0.926 -> slope 0.440, half 3*24.5
  export BASE=$T/k3_run5_C.txt CMAX=196 DMAX=412 RIDGE=0.440,74
fi
export ROWS J=6 RL=16384 NB=21 BMAX=5e3 H=4 KEEP=1 NSAVE=100000 CSTEP=2 DSTEP=4 TMPDIR_W5=$OUT
for i in 0 1 2 3; do
  CPART=$i/4 setsid nohup "$PY" scan3.py "$MODE" > "$T/k3_j6scan_${MODE}_p$i.log" 2>&1 < /dev/null &
done
