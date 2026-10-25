"""Score E = R64(tx + R64(ty - lnB)) with lnB = TOMS betaln (Excel GAMMALN/LN) per probe sheet and zone."""
import sys, collections, mpmath
from lib import load_probe, mpf, D
from cscan import prep, scan
from toms_betaln import betaln
for sh in sys.argv[1:]:
    rows=[r for r in load_probe('probe1') if r['case']==sh]
    L=betaln(rows[0]['a'], rows[0]['b'])
    pre=prep(rows, lnb_override=mpf(L))
    (c,h),=scan(pre,[0.0])
    n=collections.Counter(p[2] for p in pre)
    pre0=prep(rows); (c0,h0),=scan(pre0,[0.0])
    print(sh, rows[0]['a'], rows[0]['b'], 'TOMS lnB:', sum(h.values()), dict(h), '| L1d:', sum(h0.values()), '| n', dict(n))
