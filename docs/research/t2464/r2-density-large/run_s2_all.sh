#!/bin/sh
# Free-constant structure ceiling on every fixed-shape block (s2_scan, BEST_X knobs, stored E).
cd "$(dirname "$0")"
PY=${PYTHON:-../../../../bindings/python/.venv/bin/python}
for b in "$@"; do $PY s2_scan.py "$b" "ar=x,lnmode=xl,addar=e,cut=0.2,sar=e,xar=e,comb=x" "ar=m,lnmode=m,cut=0.1,comb=x"; done
