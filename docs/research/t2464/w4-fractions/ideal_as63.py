"""Idealized tie response of AS 63 (Majumder-Bhattacharjee 1973) with cx = 1-x as an independent input."""
from ideal_c import *
def as63(p,q,x,cx,lnB):
    psq=p+q; xx=x; pp=p; qq=q
    term=M(1); ai=M(1); value=M(1); ns=int(qq+cx*psq)
    rx=xx/cx; temp=qq-ai
    if ns==0: rx=xx
    for _ in range(5000):
        term=term*temp*rx/(pp+ai); value+=term
        if abs(term)<=M(10)**-45*abs(value): break
        ai+=1; ns-=1
        if ns>=0:
            temp=qq-ai
            if ns==0: rx=xx
        else:
            temp=psq; psq+=1
    return value*mp.exp(pp*mp.log(xx)+(qq-1)*mp.log(cx)-lnB)/pp
for a,b,x,c in EXCEL:
    a,b,x=M(a),M(b),M(x); y=1-x; h=M(10)**-20; lnB=mp.log(mp.beta(a,b))
    f=lambda yy: as63(a,b,x,yy,lnB)
    f2=lambda yy: as63(a,b,x,yy,lnB)/yy**(b-1)  # series only, prefactor y^(b-1) via exact
    print(float(a),float(b),float(x),c,'full %+.2f'%float((mp.log(f(y*(1+h)))-mp.log(f(y)))/h),
          'series-only %+.2f'%(float((mp.log(f(y*(1+h)))-mp.log(f(y)))/h)-float(b-1)), 'chk %.1e'%float(f(y)/I(a,b,x)-1), 'ns',int(b+y*(a+b)))
