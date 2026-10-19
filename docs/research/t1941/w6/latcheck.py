# T-1941 W6: sanity check of a lattice result ("rec  m:" line). Prints the per-line frequency shift it implies (in DFT
# bins of a 65,536-double run; > ~0.5 bins means the offsets moved the line off the data) and, given the truth chain,
# the true offsets and the per-coefficient error. Usage: latcheck.py CAND PHASES LATOUT [TRUTH]
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from latsolve import freq_rows, ulp; from synth import read_chain
Nc,Dc=read_chain(sys.argv[1]); Nc=[v/Dc[0] for v in Nc]; Dc=[v/Dc[0] for v in Dc]
m=eval([l for l in open(sys.argv[3]) if l.startswith('rec  m:')][0].split(':',1)[1])
meas=[l.split() for l in open(sys.argv[2])]; meas=[q for q in meas if 0.7<float(q[6])<1.4]
worst={}
for q in meas:
    g=freq_rows(Nc,Dc,[(q[2],q[3],int(q[4]))],[float(q[1])])[0]
    k='%s.%s%s'%(q[2],q[3],q[4]); worst[k]=max(worst.get(k,0),abs(float(sum(a*b for a,b in zip(g,m))))*65536)
print('max |dF| (bins) per line:',' '.join('%s %.2f'%kv for kv in sorted(worst.items())))
if len(sys.argv)>4:
    Nt,Dt=read_chain(sys.argv[4]); Nt=[v/Dt[0] for v in Nt]; Dt=[v/Dt[0] for v in Dt]
    mt=[int(round((mp.mpf(a)-mp.mpf(b))/ulp(b))) for a,b in zip(Nt+Dt[1:],Nc+Dc[1:])]
    print('true m:',mt); print('err   :',[a-b for a,b in zip(m,mt)])
