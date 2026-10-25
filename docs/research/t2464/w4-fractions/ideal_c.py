"""Idealized tie response c = d ln F / d ln y (x fixed) for fraction structures, exact arithmetic (mpmath).
Compared with Excel's measured c (w2-forensics tie_response, table 2.1)."""
import mpmath as mp
mp.mp.dps = 50
M = mp.mpf
EXCEL = [(2.5,3.5,0.2114,-3.72),(2.5,3.5,0.3314,-1.36),(2.5,3.5,0.2857,-1.91),(7.3,11.6,0.2731,-8.51),
         (4,4,0.2317,-7.17),(100,120,0.4013,-10.58),(0.5,6.25,0.0431,-0.20),(1.5,0.5,0.3511,0.15),(0.5,0.7,0.4167,0.02)]

def incbcf(a,b,x,n=400):
    # A&S 26.5.8 evaluated backward exactly: 1/(1+d1/(1+d2/(1+...)))
    f = M(1)
    for m in range(n,0,-1):
        k = m//2
        if m % 2 == 1:
            d = -(a+k)*(a+b+k)*x/((a+2*k)*(a+2*k+1))
        else:
            d = k*(b-k)*x/((a+2*k-1)*(a+2*k))
        f = 1 + d/f
    return 1/f

def incbd(a,b,z,n=400):
    # Cephes incbd: CF in z = x/(1-x). coefficient pattern from cephes (k2=b-1 decreasing, k6=a+b increasing)
    # evaluate forward like cephes with exact arithmetic
    k1,k2,k3,k4,k5,k6,k7,k8 = a, b-1, a, a+1, M(1), a+b, a+1, a+2
    pkm2,qkm2,pkm1,qkm1 = M(0),M(1),M(1),M(1)
    for _ in range(n):
        xk = -(z*k1*k2)/(k3*k4); pk=pkm1+pkm2*xk; qk=qkm1+qkm2*xk; pkm2,pkm1,qkm2,qkm1=pkm1,pk,qkm1,qk
        xk = (z*k5*k6)/(k7*k8); pk=pkm1+pkm2*xk; qk=qkm1+qkm2*xk; pkm2,pkm1,qkm2,qkm1=pkm1,pk,qkm1,qk
        k1+=1;k2-=1;k3+=2;k4+=2;k5+=1;k6+=1;k7+=2;k8+=2
    return pk/qk

def bfrac(a,b,x,y,lam,n=400):
    c = lam + 1; c0 = b/a; c1 = 1 + 1/a; yp1 = y + 1
    p = M(1); s = a + 1; an = M(0); bn = M(1); anp1 = M(1); bnp1 = c/c1; r = c1/c
    for k in range(1,n):
        t = k/a; w = k*(b-k)*x; e = a/s; alpha = p*(p+c0)*e*e*(w*x)
        e = (1+t)/(c1+t+t); beta = k + w/s + e*(c + k*yp1); p = 1+t; s += 2
        t = alpha*an + beta*anp1; an = anp1; anp1 = t
        t = alpha*bn + beta*bnp1; bn = bnp1; bnp1 = t
        r = anp1/bnp1
    return r   # bfrac returns brc*r

def I(a,b,x): return mp.betainc(a,b,0,x,regularized=True)

def struct(name, a,b,x,y):
    lnB = mp.log(mp.beta(a,b))
    if name=='Py*incbcf': return x**a*y**b/(a*mp.beta(a,b))*incbcf(a,b,x)
    if name=='P0*incbcf': return x**a*(1-x)**b/(a*mp.beta(a,b))*incbcf(a,b,x)
    if name=='Py*incbd/y': return x**a*y**b/(a*mp.beta(a,b))*incbd(a,b,x/y)/y
    if name=='P0*incbd/y': return x**a*(1-x)**b/(a*mp.beta(a,b))*incbd(a,b,x/y)/y
    if name=='P0*incbd/(1-x)': return x**a*(1-x)**b/(a*mp.beta(a,b))*incbd(a,b,x/y)/(1-x)
    if name=='Py*bfrac(lamx)': return x**a*y**b/mp.beta(a,b)*bfrac(a,b,x,y,a-(a+b)*x)
    if name=='P0*bfrac(lamx)': return x**a*(1-x)**b/mp.beta(a,b)*bfrac(a,b,x,y,a-(a+b)*x)
    if name=='P0*bfrac(lamy)': return x**a*(1-x)**b/mp.beta(a,b)*bfrac(a,b,x,y,(a+b)*y-b)
    if name=='Py*bfrac(lamy)': return x**a*y**b/mp.beta(a,b)*bfrac(a,b,x,y,(a+b)*y-b)
    if name=='1-I_y(b,a)': return 1-I(b,a,y)
    if name=='1-Py(b,a)*incbcf(y)': return 1 - y**b*x**a/(b*mp.beta(a,b))*incbcf(b,a,y)
    raise KeyError
NAMES=['Py*incbcf','Py*incbd/y','P0*incbd/y','Py*bfrac(lamx)','P0*bfrac(lamx)','P0*bfrac(lamy)','Py*bfrac(lamy)','1-I_y(b,a)']
print('a b x excel | ' + ' | '.join(NAMES))
for a,b,x,c in EXCEL:
    a,b,x = M(a),M(b),M(x); y=1-x
    # sanity
    out=[]
    for nm in NAMES:
        h = M(10)**-20
        f0 = struct(nm,a,b,x,y); f1 = struct(nm,a,b,x,y*(1+h))
        out.append(float((mp.log(f1)-mp.log(f0))/h))
    chk = [float(struct(nm,a,b,x,y)/I(a,b,x)-1) for nm in NAMES]
    print(f"{float(a)} {float(b)} {float(x)} {c:+.2f} | " + ' '.join(f"{v:+7.2f}" for v in out), ' maxchk %.1e'%max(abs(t) for t in chk))
