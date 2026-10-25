"""Positive control for the offset scan: synthetic 'Excel' = M1 with lnB moved by +3.0 units (2^-53) on probe1C6
and probe1C0 (even k). cscan must recover c = -3.0 at 100% hits (and c = 0 must not)."""
import collections, numpy as np
from lib import load_probe, mpf, exp_v6, R64
from cscan import prep, scan
from model_final import lnB_model
U=mpf(2)**-53
for sh in ('probe1C6','probe1C0'):
    rows=[dict(r) for r in load_probe('probe1') if r['case']==sh and r['k']%2==0]
    L=lnB_model(rows[0]['a'],rows[0]['b'])
    pre=prep(rows, lnb_override=L)
    # synthetic: E_true = E - 3U (lnB larger by 3 units)
    syn=[(E, exp_v6(R64(E-3*U)), z, r) for E,v,z,r in pre]
    res=scan(syn, np.arange(-4,4.01,0.25))
    best=max(res,key=lambda t:sum(t[1].values()))
    at0=[t for t in res if abs(t[0])<1e-9][0]
    print(sh, 'recovered c', best[0], sum(best[1].values()),'/',len(syn), '| at 0:', sum(at0[1].values()))
