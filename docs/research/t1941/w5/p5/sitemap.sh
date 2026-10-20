#!/bin/sh
# Per-site mismatch map: APROFILE at b = c = 0 for every stage (Z at a = 0 strongly negative = the base's own
# rounding pattern at that site is missing from Excel). usage: sitemap.sh CHAIN "J list" OUT
cd "$(dirname "$0")"
S=/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad
for j in $2; do
  echo "== $1.S$j"; ROWS=$S/rows6.txt RL=16384 APROFILE=1 CHAIN=$1 J=$j python3 scan2.py real 2>&1 | awk '/S null/ || /a 0\.(000|125|250|375|500|625|750|875) /'
done > $3 2>&1
