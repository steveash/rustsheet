#!/bin/sh
# wait until the real wide N.S7 scan or the far-truth control finishes, then show both logs
cd "$(dirname "$0")"
while pgrep -f "python3 scan2.py real" >/dev/null && pgrep -f "python3 scan2.py control" >/dev/null; do sleep 60; done
for f in scan_real_NS7_wide.log scan_farctl_NS7.log; do echo "== $f"; tail -n 15 $f; done
