#!/bin/sh
# 1-D generator-fingerprint solves: hold a, solve b so that R0 matches Excel; 4 jobs at a time.
cd "$(dirname "$0")"
for job in "A 8 8 rel 0.7 1.6" "A 9 8 rel 0.7 1.8" "g 8 8 rel 0.7 1.8" "g 8 9 rel 0.7 1.9" \
           "A 8 8 rel 0.5 1.6" "A 9 8 rel 0.5 1.8" "g 8 8 rel 0.5 1.8" "A 8 8 absA 0.7 1.6" \
           "g 8 8 absg 0.7 1.8" "A 8 9 rel 0.7 1.9" "A 7 7 rel 0.7 1.3" "A 9 9 rel 0.7 2.0"; do
  while [ "$(pgrep -fc '[p]ython3 solve2')" -ge 4 ]; do sleep 5; done
  nohup python3 solve2.py $job a > "fix_$(echo $job | tr ' ' _).txt" 2>&1 &
  sleep 1
done
wait
