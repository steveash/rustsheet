#!/bin/sh
# T-2122 run 4 (HANDOFF2 16): how noisy is the refit protocol of 15.3? Perturb N6 by d ulps on top of the head+N4-N6
# move and on top of the zero move, refit N0-N3 + D1-D9 to ss_fc's function, score all. usage: k3_refit_noise.sh PY ROWS.npz TMP
PY=$1; ROWS=$2; T=$3; B=w5/p4/ss_fc.txt; L=""
for d in 0 1 10 100 1000 10000 100000; do
  $PY t2122/k3_move.py w5 $B $T/p$d.txt N7=4.007e8 N8=-4.611e10 N9=3.503e12 N6=$((736800000+d)) N5=-1.603e9 N4=3.869e7
  $PY t2122/k3_move.py w5 $B $T/z$d.txt N6=$d
  $PY w5/p5/refit_fn.py $T/p$d.txt $B $T/p${d}_j4.txt N 4 "" >/dev/null
  $PY w5/p5/refit_fn.py $T/z$d.txt $B $T/z${d}_j4.txt N 4 "" >/dev/null
  L="$L $T/p${d}_j4.txt $T/z${d}_j4.txt"
done
$PY t2122/k3_score.py w5 $ROWS $L
