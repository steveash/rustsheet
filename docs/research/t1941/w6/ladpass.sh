# T-1941 W6: one lattice pass from measured inputs: clean 24-run phases + ladder frequency rows + frequency hold.
# Usage: ladpass.sh RFILE CHAIN TAG OUTDIR LADSRC [TRUTH]
#   LADSRC "synth": simulate both ladder sets from TRUTH; otherwise an RFILE holding Excel's ladder rows (rext.py format)
set -e
cd "$(dirname "$0")/.."
R=$1; C=$2; T=$3; O=$4; LD=$5; TR=$6
python3 w6/xlphase.py $R $C $O/${T}_ph.txt > /dev/null
python3 w6/selclean.py $O/${T}_ph.txt $O/${T}_phc.txt
: > $O/${T}_lad.txt
for SET in ${LADSETS:-dyadic nd}; do
  if [ "$LD" = synth ]; then python3 w6/ladder.py synth $SET $TR $C $O/${T}_lad_$SET.txt > $O/${T}_lad_$SET.log
  else python3 w6/ladder.py excel $SET $LD $C $O/${T}_lad_$SET.txt > $O/${T}_lad_$SET.log; fi
  cat $O/${T}_lad_$SET.txt >> $O/${T}_lad.txt
done
LD=$O/${T}_lad.txt
LADDER=$LD LADRES=${LADRES:-0.15} LADW=${LADW:-3e8} FREQ=${FREQ:-2621} PHASES=$O/${T}_phc.txt BKZ=${BKZ:-30} MPFR=200 python3 w6/latsolve.py $C $C measured 0.02 15 > $O/${T}_lat.txt
grep -E "rows" $O/${T}_lat.txt
python3 w6/latcheck.py $C $O/${T}_phc.txt $O/${T}_lat.txt $TR | sed 's/^/  /'
python3 w6/applym.py $C $O/${T}_lat.txt $O/${T}_chain.txt
echo "wrote $O/${T}_chain.txt"
