#!/bin/sh
# End-to-end control for the N chain below the top: refit, then peel N6..N0 (each step: exact two-site scan, apply).
cd "$(dirname "$0")"
python3 refit_fn.py e2e_top.txt e2e_truth.txt e2e_n7.txt N 7 N0,N1,N2,D1,D2,D3,D9
cur=e2e_n7.txt
for j in 6 5 4 3 2 1 0; do
  nxt=e2e_n$j.txt
  BASE=$cur TRUTHFILE=e2e_truth.txt KMAX=${KMAX:-1e5} APPLY=1 python3 peel2.py control N $j $nxt | tail -3
  cur=$nxt
done
