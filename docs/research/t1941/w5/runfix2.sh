#!/bin/sh
cd "$(dirname "$0")"
while pgrep -f '[r]unfix.sh' >/dev/null; do sleep 10; done
for job in "A 8 8 rel 0.75 1.55" "A 8 8 rel 0.65 1.75" "A 8 8 rel 0.8 1.5" "A 8 8 rel 0.6 1.8"; do
  while [ "$(pgrep -fc '[p]ython3 solve2')" -ge 4 ]; do sleep 5; done
  nohup python3 solve2.py $job a > "fix_$(echo $job | tr ' ' _).txt" 2>&1 &
  sleep 1
done
wait
