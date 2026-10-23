#!/bin/sh
# Free-constant structure ceiling on every fixed-shape block (s2_scan, BEST_X knobs, stored E).
PY=/home/user/rustsheet/bindings/python/.venv/bin/python
cd "$(dirname "$0")"
for b in "$@"; do $PY s2_scan.py "$b" "ar=x,lnmode=xl,addar=e,cut=0.2,sar=e,xar=e,comb=x" "ar=m,lnmode=m,cut=0.1,comb=x"; done
