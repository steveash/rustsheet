"""Compare TOMS 708 betaln (Excel GAMMALN/LN inside, or cr variants) with L1d and the cscan2 best offsets."""
import math, mpmath
from lib import D, mpf, xl
from toms_betaln import betaln, GL, LN
U=2.0**-53
best={'C0':(0.5,7,-0.05),'C3':(0.25,3.3,0.0),'C4':(2.5,3.5,6.0),'C6':(0.5,20,12.0),'C7':(3,0.5,0.0),'C9':(1,20,3.8),'C2':(1,7,None),'C5':(12,20,56)}
crlog=lambda v: float(mpmath.log(mpf(v)))
crgl=lambda v: float(mpmath.loggamma(mpf(v)))
for k,(a,b,c) in best.items():
    a=float(a); b=float(b)
    L1d=float(D(mpf(GL(a))+D(mpf(GL(b))-GL(a+b))))
    ex=float(mpmath.log(mpmath.beta(a,b)))
    t1=betaln(a,b); t2=betaln(a,b,gamln=crgl,log=crlog)
    print(k,a,b,'needed c',c,'| L1d-exact %.2f'%((L1d-ex)/U), '| L1d-toms(Excel GL,LN) %.2f'%((L1d-t1)/U), '| L1d-toms(cr) %.2f'%((L1d-t2)/U), '| L1d - (-LN b) %.2f'%((L1d+LN(b))/U) if a==1 else '')
