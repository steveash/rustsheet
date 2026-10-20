# T-1941 W6: line selection for the measured-phase lattice. Per line: median |log amplitude ratio| (data/candidate
# DFT at the predicted frequency), the selector usable on Excel; with a truth chain, also the median error of the
# measured phase against the ideal (truth - candidate) phase. Usage: phasediag.py PHASES CAND [TRUTH]
import sys, collections, numpy as np
sys.path.insert(0,sys.path[0]); from latsolve import phase_rows; from synth import read_chain, nrm
def norm(f): N,D=read_chain(f); return [v/nrm(D) for v in N],[v/nrm(D) for v in D]
Nc,Dc=norm(sys.argv[2]); T=norm(sys.argv[3]) if len(sys.argv)>3 else None
by=collections.defaultdict(list)
for l in open(sys.argv[1]):
    q=l.split(); ln=(q[2],q[3],int(q[4])); e=np.nan
    if T:
        rc=phase_rows(Nc,Dc,[ln],[float(q[1])])[0][1]; rt=phase_rows(T[0],T[1],[ln],[float(q[1])])[0][1]
        e=abs((float(q[5])-float((rt-rc)%1)+0.5)%1-0.5)
    by['%s.%s%d'%ln].append((abs(np.log(max(float(q[6]),1e-9))),e))
for k,v in sorted(by.items(),key=lambda kv: np.median([a for a,_ in kv[1]])):
    a=np.array(v); print('%-7s med|log r| %.3f  med phase err %.3f  n %d'%(k,np.median(a[:,0]),np.median(a[:,1]),len(v)))
