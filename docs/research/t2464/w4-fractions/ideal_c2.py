from ideal_c import *
def s2(name,a,b,x,y):
    B = mp.beta(a,b)
    if name=='1-Py(b,a)incbcf(y)': return 1 - y**b*x**a/(b*B)*incbcf(b,a,y)
    if name=='1-P0(b,a)incbcf(y)': return 1 - (1-x)**b*x**a/(b*B)*incbcf(b,a,y)
    if name=='1-Py(b,a)incbd(y/x)/x': return 1 - y**b*x**a/(b*B)*incbd(b,a,y/x)/x
    if name=='1-P0(b,a)incbd(y/x)/x': return 1 - (1-x)**b*x**a/(b*B)*incbd(b,a,y/x)/x
    if name=='1-Py(b,a)bfrac(y,x,lam(y))': return 1 - y**b*x**a/B*bfrac(b,a,y,x,b-(a+b)*y)
    if name=='1-P0(b,a)bfrac(y,x,lam(y))': return 1 - (1-x)**b*x**a/B*bfrac(b,a,y,x,b-(a+b)*y)
N2=['1-Py(b,a)incbcf(y)','1-P0(b,a)incbcf(y)','1-Py(b,a)incbd(y/x)/x','1-P0(b,a)incbd(y/x)/x','1-Py(b,a)bfrac(y,x,lam(y))','1-P0(b,a)bfrac(y,x,lam(y))']
print(N2)
for a,b,x,c in EXCEL:
    a,b,x = M(a),M(b),M(x); y=1-x; h=M(10)**-20
    out=[float((mp.log(s2(n,a,b,x,y*(1+h)))-mp.log(s2(n,a,b,x,y)))/h) for n in N2]
    chk=max(abs(float(s2(n,a,b,x,y)/I(a,b,x)-1)) for n in N2)
    print(f"{float(a)} {float(b)} {float(x)} {c:+.2f} | "+' '.join(f"{v:+7.2f}" for v in out), '%.0e'%chk)
