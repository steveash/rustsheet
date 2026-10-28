#!/bin/sh
# T-2122 run 5 (HANDOFF2 17.5): the real N.S7 box scan (w5/p5/scan2.py, S-only ranks) centred on the run-5 set C.
# The ladder's 3-sigma box from C is |b| <= 1.1e3, |c| <= 1.65e3 (k3_topbox.py); scanned c is +-1800 step 0.5,
# centre-out, in 4 resumable parts with checkpoints kept in this directory (rerun to resume; merge with
# MERGE=1 CKPTS=t2122/ck_run5_0.txt,...). usage (from docs/research/t1941): k3_topscan.sh PY ROWS
# (ROWS: the w5/p5/mkrows.py output path without its .near1.npz suffix)
PY=$1; ROWS=$2; T=$(pwd)/t2122
cd w5/p5 || exit 1
export ROWS BASE=$T/k3_run5_C.txt RL=16384 CMAX=1800 CSTEP=0.5 NB=21 BMAX=5e3 H=4 KEEP=10 CORDER=center
for i in 0 1 2 3; do
  CPART=$i/4 CKPT=$T/ck_run5_$i.txt setsid nohup "$PY" scan2.py real > "$T/k3_topscan_p$i.log" 2>&1 < /dev/null &
done
