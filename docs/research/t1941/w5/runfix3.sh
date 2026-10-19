#!/bin/sh
cd "$(dirname "$0")"
while pgrep -f '[r]unfix2.sh' >/dev/null; do sleep 10; done
for job in "g 8 8 rel 0.7 1.95" "g 8 9 rel 0.7 1.95" "g 8 8 rel 0.75 1.9" "A 8 8 absB 0.7 1.65"; do
  while [ "$(pgrep -fc '[p]ython3 solve2')" -ge 4 ]; do sleep 5; done
  nohup python3 solve2.py $job a > "fix_$(echo $job | tr ' ' _).txt" 2>&1 &
  sleep 1
done
wait
