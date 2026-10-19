# T-1941 W6: top-down peel of whole-ulp shifts: for each coefficient (given order), products-only m-scan (fscan6
# SITES=P NS=1) at the next product, apply the best k, continue. Usage: peel6.sh RFILE CHAIN OUTPREFIX "N5 N4 ..." [TRUTH]
set -e
cd "$(dirname "$0")/.."
R=$1; C=$2; O=$3; LIST=$4; TR=$5
cur=$C
for cj in $LIST; do
  ch=${cj:0:1}; j=${cj:1}
  line=$(SITES=P NS=1 NPTS=${NPTS:-3000000} python3 w6/fscan6.py $R $cur $ch $j ${KR:-1e4} $TR | tee /dev/stderr | head -1)
  k=$(echo "$line" | sed -E 's/.*top: k=([+-][0-9]+).*/\1/')
  python3 w6/bump.py $cur ${O}_$cj.txt $cj=$k
  echo "$cj: k $k -> ${O}_$cj.txt"
  cur=${O}_$cj.txt
done
