#!/bin/bash
# T-1941 W6: end-to-end pipeline on Excel's long runs kmyst-gam-L101..L124. Usage (repo root): run_L.sh SCRATCH
# 1) rows -> per-point R; 2) precise per-run peaks; 3) model-free track growing; 4) per-track law candidates for the
# 12 longest tracks (exponent offsets -2..2); then filter law tracks (best rms < 2e-5) and joint beam + EM by hand.
set -e
S=$1; W=docs/research/t1941/w6
python3 $W/rows_all.py $S/rowsL.txt 1.5 2.0
(cd docs/research/t1941 && python3 w6/rext.py $S/rowsL.txt $S/RL.txt)
(cd docs/research/t1941 && python3 w6/longspec.py excel $S/RL.txt $S/Lxl.npz)
(cd docs/research/t1941 && python3 w6/grow.py $S/Lxl.npz $S/Lxl_tracks.json 4e-4 7)
cd docs/research/t1941
for ti in $(seq 0 11); do
  NCAND=20 timeout 7000 python3 w6/enumlong.py $S/Lxl_tracks.json $ti 9 100 $S/Lxcand_$ti.json -2,-1,0,1,2 T,S,Pn1 > $S/Lxenum_$ti.txt 2>&1 &
  if (( (ti+1) % 4 == 0 )); then wait; fi
done
wait
for ti in $(seq 0 11); do echo "track $ti: $(head -1 $S/Lxenum_$ti.txt)"; done
