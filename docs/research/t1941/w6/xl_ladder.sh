# T-1941 W6: Excel ladder readout once q421-q433 / q440-q453 are filled. Dumps Excel's inner R for (1.5,2) (rows_all.py
# + rext.py, scratch only) and reads both ladder sets against CHAIN. Usage: xl_ladder.sh CHAIN OUTDIR
set -e
cd "$(dirname "$0")/../../../.."
C=$1; O=$2
python3 docs/research/t1941/w6/rows_all.py $O/rows_all.txt 1.5 2
python3 docs/research/t1941/w6/rext.py $O/rows_all.txt $O/Rall.txt
for SET in dyadic nd; do
  python3 docs/research/t1941/w6/ladder.py excel $SET $O/Rall.txt $C $O/xl_lad_$SET.txt > $O/xl_lad_$SET.log || true
  echo "$SET: $(grep -c F_meas $O/xl_lad_$SET.log) lines read, $(awk '$NF<0.15' $O/xl_lad_$SET.log | grep -c F_meas) pass the 0.15 gate, $(grep -c missing $O/xl_lad_$SET.log) missing runs"
  awk '$NF<0.15 {print "   ",$1,$2}' $O/xl_lad_$SET.log | sort | uniq | tr '\n' ' '; echo
done
