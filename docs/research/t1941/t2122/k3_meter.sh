#!/bin/sh
# T-2122 run 8 (HANDOFF2 20): the bias subtraction of run 7 (scan3.py BIASFILE/BIASKN) as a store-structure meter. The
# J=6 scan of the real data from C with the base's own residual under an 80-bit top (ops j >= BIASKN) subtracted, on a
# small box around the base (|c| <= 60, |d| <= 60, c step 2, d step 4; 1,891 pairs) that holds the base cluster, the
# 18.3 ridge out to |d| 60 and the nearest mirror sidelobes (b ~12, c ~-44, d ~52). The right BIASKN should leave a
# compact base cluster, as KN = 7 does on its control (19.1); a wrong one leaves the ridge and the sidelobes (19.2).
# usage (from docs/research/t1941): k3_meter.sh PY ROWS BIASKN OUTDIR [real|control CTLKN]
#   control: data synthesised from k3_run5_ctl_truth.txt with ops j >= CTLKN in 80-bit, base and bias c7.
# Runs the four parts in the foreground (about 5 min on 4 cores) and logs to t2122/k3_meter_<mode><ctlkn>_kn<BIASKN>_p*.log.
PY=$1; ROWS=$2; BKN=$3; OUT=$4; MODE=${5:-real}; CKN=$6; T=$(pwd)/t2122
mkdir -p "$OUT"
cd w5/p5 || exit 1
if [ "$MODE" = control ]; then
  export BASE=$T/k3_run5_ctl_c7.txt TRUTHFILE=$T/k3_run5_ctl_truth.txt KN=$CKN BIASFILE=$T/k3_run5_ctl_c7.txt
else
  export BASE=$T/k3_run5_C.txt BIASFILE=$T/k3_run5_C.txt
fi
export BIASKN=$BKN ROWS J=6 RL=16384 NB=21 BMAX=5e3 H=4 KEEP=1 NSAVE=100000 CSTEP=2 DSTEP=4 CMAX=60 DMAX=60
export TMPDIR_W5=$OUT
for i in 0 1 2 3; do
  CPART=$i/4 "$PY" scan3.py "$MODE" > "$T/k3_meter_${MODE}${CKN}_kn${BKN}_p$i.log" 2>&1 < /dev/null &
done
wait
