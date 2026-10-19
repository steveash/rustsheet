# T-1941 W6: keep the measured-phase rows of "clean" lines: a line's median |log amplitude ratio| < THR (default 0.1)
# and the row's ratio in [0.7, 1.4]. On the far control this keeps every accurate line except N.Pn12 and no junk line
# (README item 24). Usage: selclean.py PHASES OUT [THR]
import sys, collections, numpy as np
rows=[l.split() for l in open(sys.argv[1])]; thr=float(sys.argv[3]) if len(sys.argv)>3 else 0.1
by=collections.defaultdict(list)
for q in rows: by[(q[2],q[3],q[4])].append(abs(np.log(max(float(q[6]),1e-9))))
good={k for k,v in by.items() if np.median(v)<thr}
with open(sys.argv[2],'w') as o:
    for q in rows:
        if (q[2],q[3],q[4]) in good and 0.7<float(q[6])<1.4: o.write(' '.join(q)+'\n')
print('clean lines:',' '.join(sorted('%s.%s%s'%k for k in good)))
