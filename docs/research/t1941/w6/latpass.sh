# T-1941 W6: one measured-phase lattice pass with frequency-hold rows. Usage: latpass.sh RFILE CHAIN TAG OUTDIR
set -e
cd "$(dirname "$0")/.."
R=$1; C=$2; T=$3; O=$4
python3 w6/xlphase.py $R $C $O/${T}_ph.txt > /dev/null
awk '{r=$7; k=$3"."$4$5; n[k]++; if(r>0.7&&r<1.4) ok[k]++} END{for(k in n) printf "%s %d/%d  ", k, ok[k], n[k]; print ""}' $O/${T}_ph.txt
FREQ=${FREQ:-2621} PHASES=$O/${T}_ph.txt BKZ=${BKZ:-30} MPFR=200 python3 w6/latsolve.py $C $C measured 0.02 15 > $O/${T}_lat.txt
python3 w6/latcheck.py $C $O/${T}_ph.txt $O/${T}_lat.txt
python3 w6/applym.py $C $O/${T}_lat.txt $O/${T}_chain.txt
echo "wrote $O/${T}_chain.txt"
