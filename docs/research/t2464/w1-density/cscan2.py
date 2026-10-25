"""Wide constant-offset scan per sheet (E extended, brcomp logs): Excel's effective lnB = L1d - c*2^-53.
Coarse scan +-60 units, then fine around the best. Prints best c and hit counts."""
import sys, collections, numpy as np
from lib import load_probe, mpf
from cscan import prep, scan
for sh in sys.argv[1:]:
    rows=[r for r in load_probe('probe1') if r['case']==sh and r['k']%2==0]   # tune on even k
    pre=prep(rows)
    res=scan(pre, np.arange(-60,60.01,0.5))
    best=max(res,key=lambda t:sum(t[1].values()))
    res2=scan(pre, np.arange(best[0]-0.6,best[0]+0.61,0.05))
    b2=max(res2,key=lambda t:sum(t[1].values()))
    # hold-out: odd k
    rows_o=[r for r in load_probe('probe1') if r['case']==sh and r['k']%2==1]
    ho=scan(prep(rows_o),[b2[0], 0.0])
    print(sh, rows[0]['a'], rows[0]['b'], 'best c %.2f'%b2[0], 'tune', sum(b2[1].values()),'/',len(pre), dict(b2[1]), '| holdout at c:', sum(ho[0][1].values()), 'at 0:', sum(ho[1][1].values()), '/', len(rows_o))
