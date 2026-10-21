"""T-2142 fit: ATAN2 candidates against the Mac desktop cells of ATAN2_POWER.xlsx and the web probes.

Run from the repo root (decision 0086; numbers.md "Excel for the web and desktop Excel differ in the last digits"):
    target/release/rustsheet-eval snapshot \
        crates/rustsheet-xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/ATAN2_POWER.xlsx > /tmp/atan2.json
    cd tools && uv run python ../docs/research/t2142/fit_atan2.py .. /tmp/atan2.json
"""
import json, re, sys
WT=sys.argv[1]

def num(v):
    if isinstance(v,bool): return float(v)
    if isinstance(v,(int,float)): return float(v)
    return None
def desktop():
    d=json.load(open(sys.argv[2]))['sheets'][0]['cells']
    out=[]
    for r in range(2,29):
        c=d[f'D{r}']['value']
        if c['t']!='n': continue
        def val(k):
            e=d.get(k)
            if e is None: return 0.0
            v=e['value']; return float(v['v']) if v['t'] in('n','b') else None
        out.append(('desk',f'D{r}',val(f'A{r}'),val(f'B{r}'),c['v']))
    return out
def web():
    out=[]
    for name in ['atan2-quadrants','atan2-error-edges','atan2-kept-subnormal','asin-atan-sign-source']:
        case=json.load(open(f'{WT}/eval/suites/probes/cases/{name}.case.json'))
        orc=json.load(open(f'{WT}/eval/suites/probes/cases/{name}.oracle.json'))['sheets']
        for sh in case['sheets']:
            cells=sh['cells']; o=orc.get(sh['name'],{})
            for k,f in cells.items():
                if not isinstance(f,str): continue
                m=re.fullmatch(r'=ATAN2\(([A-Z]+\d+),([A-Z]+\d+)\)',f)
                if not m: continue
                a,b=num(cells.get(m[1],0.0)) if m[1] in cells else 0.0, num(cells.get(m[2],0.0)) if m[2] in cells else 0.0
                if a is None or b is None: continue
                ov=o.get(k)
                if not ov or ov['t']!='n': continue
                out.append((name,k,a,b,ov['v']))
    return out
import sys, math, decimal
from fractions import Fraction as F
from rustsheet_tools.fitlab.exact import round_to_bits, sqrt_cr
D=decimal.Context(prec=90)
def dec(f): f=F(f); return D.divide(decimal.Decimal(f.numerator),decimal.Decimal(f.denominator))
def atan_d(x):
    x=dec(x); k=0
    while abs(x)>decimal.Decimal('0.05'):
        x=D.divide(x, D.add(1, D.sqrt(D.add(1, D.multiply(x,x))))); k+=1
    s=decimal.Decimal(0); t=x; n=1; x2=D.multiply(x,x); sign=1
    while True:
        term=D.divide(t,n)
        if abs(term)<=abs(x)*decimal.Decimal('1e-85'): break
        s=D.add(s, term if sign>0 else -term); t=D.multiply(t,x2); n+=2; sign=-sign
    return F(s*(2**k))
PI_D=F(atan_d(1)*4)
def rn(x,p=53): return float(round_to_bits(x,p,-1074 if p==53 else None))
def x87(x): return rn(round_to_bits(x,64),53)
def xdiv(y,x): return x87(F(y)/F(x))
def xmul(a,b): return x87(F(a)*F(b))
def xadd(a,b): return x87(F(a)+F(b))
def xsqrt(a): return rn(sqrt_cr(F(a),64) if True else 0)
PI=math.pi
def adjust(a,x,y,add=lambda a,b:a+b):
    if x<0: return add(a,PI) if y>=0 else add(a,-PI)
    return a+0.0
# fdlibm atan
atanhi=[4.63647609000806093515e-01,7.85398163397448278999e-01,9.82793723247329054082e-01,1.57079632679489655800e+00]
atanlo=[2.26987774529616870924e-17,3.06161699786838301793e-17,1.39033110312309984516e-17,6.12323399573676603587e-17]
aT=[3.33333333333329318027e-01,-1.99999999998764832476e-01,1.42857142725034663711e-01,-1.11111104054623557880e-01,9.09088713343650656196e-02,-7.69187620504482999495e-02,6.66107313738753120669e-02,-5.83357013379057348645e-02,4.97687799461593236017e-02,-3.65315727442169155270e-02,1.62858201153657823623e-02]
def fd_atan(x):
    sgn=x<0; x=abs(x)
    if x>=4.4028257629870e15: r=atanhi[3]+atanlo[3]; return -r if sgn else r
    if x<0.4375:
        if x<2**-27: return -x if sgn else x
        id=-1
    else:
        if x<1.1875:
            if x<0.6875: id=0; x=(2.0*x-1.0)/(2.0+x)
            else: id=1; x=(x-1.0)/(x+1.0)
        else:
            if x<2.4375: id=2; x=(x-1.5)/(1.0+1.5*x)
            else: id=3; x=-1.0/x
    z=x*x; w=z*z
    s1=z*(aT[0]+w*(aT[2]+w*(aT[4]+w*(aT[6]+w*(aT[8]+w*aT[10])))))
    s2=w*(aT[1]+w*(aT[3]+w*(aT[5]+w*(aT[7]+w*aT[9]))))
    if id<0: r=x-x*(s1+s2)
    else: r=atanhi[id]-((x*(s1+s2)-atanlo[id])-x)
    return -r if sgn else r
def cr_atan(q): return rn(atan_d(q))
cands={
 'engine: CR atan(q), ±pi in doubles': lambda x,y: adjust(cr_atan(xdiv(y,x)),x,y),
 'x87: RN53(RN64(atan q)), ±pi in doubles': lambda x,y: adjust(x87(atan_d(xdiv(y,x))),x,y),
 'FPATAN(y,x) exact args, RN64 then RN53': lambda x,y: x87(atan_d(F(y)/F(x)) + (0 if x>0 else (PI_D if y>=0 else -PI_D))),
 'fdlibm atan(q)': lambda x,y: adjust(fd_atan(xdiv(y,x)),x,y),
 'pi/2 - atan(1/q) for |q|>1 (doubles)': lambda x,y: adjust((lambda q: math.copysign(PI/2,q)-cr_atan(1/q) if abs(q)>1 else cr_atan(q))(xdiv(y,x)),x,y),
 'asin(q/sqrt(1+q*q))': lambda x,y: adjust((lambda q: rn(F(0)) if False else rn(asin_d(F(q)/F(math.sqrt(1+q*q)))) if abs(q)<1e150 else math.copysign(PI/2,q))(xdiv(y,x)),x,y),
 '2 atan(q/(1+sqrt(1+q*q)))': lambda x,y: adjust((lambda q: 2*cr_atan(q/(1+math.sqrt(1+q*q))) if abs(q)<1e150 else math.copysign(PI/2,q))(xdiv(y,x)),x,y),
 'q rounded to 64 bits only, CR atan': lambda x,y: adjust(rn(atan_d(round_to_bits(F(y)/F(x),64))),x,y),
 'x87 ±pi: RN53(RN64(atan q)+pi64)': lambda x,y: (lambda a: x87(a) if x>0 else x87(round_to_bits(a,64)+round_to_bits(PI_D,64)*(1 if y>=0 else -1)))(round_to_bits(atan_d(xdiv(y,x)),64)),
}
cands.update({
 'pi/4 + atan((q-1)/(q+1)) for |q|>0.4375 (doubles)': lambda x,y: adjust((lambda q: (PI/4+cr_atan((q-1)/(q+1))) if q>0.4375 else (-PI/4+cr_atan((q+1)/(1-q)) if q<-0.4375 else cr_atan(q)))(xdiv(y,x)),x,y),
 'pi/2 - atan(1/q) all x87, |q|>1': lambda x,y: adjust((lambda q: x87(round_to_bits(PI_D/2,64)*(1 if q>0 else -1) - round_to_bits(atan_d(round_to_bits(F(1)/F(q),64)),64)) if abs(q)>1 else cr_atan(q))(xdiv(y,x)),x,y),
})
def asin_d(v):
    v=F(v)
    if abs(v)>=1: return PI_D/2*(1 if v>0 else -1)
    s=D.sqrt(D.subtract(1,D.multiply(dec(v),dec(v))))
    return atan_d(F(dec(v)/s))
def sign_source():
    import json
    c=json.load(open(WT+'/eval/suites/probes/cases/asin-atan-sign-source.case.json'))['sheets'][0]['cells']
    o=json.load(open(WT+'/eval/suites/probes/cases/asin-atan-sign-source.oracle.json'))['sheets']['Sheet1']
    out=[]
    for r in range(1,200):
        if f'G{r}' not in c or not isinstance(c.get(f'A{r}'),(int,float)): continue
        a=float(c[f'A{r}']); ov=o.get(f'G{r}')
        if not ov or ov['t']!='n': continue
        x=x87(sqrt_cr(round_to_bits(F(xadd(1,-a))*F(xadd(1,a)),64),64)) if False else rn(sqrt_cr(F(xmul(xadd(1,-a),xadd(1,a))),64))
        out.append(('sign-source G',f'G{r}',x,a,ov['v']))
    return out
pts=desktop()+web()+sign_source()
eng=cands['engine: CR atan(q), ±pi in doubles']
for name,f in cands.items():
    cnt={}
    miss=[]
    for src,k,x,y,want in pts:
        if x==0: got=math.copysign(PI/2,y)
        else:
            try: got=f(x,y)
            except Exception as e: got=None
        g='desk' if src=='desk' else 'web'
        ok=got==want
        cnt.setdefault(g,[0,0]); cnt[g][0]+=ok; cnt[g][1]+=1
        if not ok and src=='desk': miss.append(k)
    print(f'{name}: desktop {cnt["desk"][0]}/{cnt["desk"][1]}, web {cnt["web"][0]}/{cnt["web"][1]}; desk misses {miss}')
print('engine web misses:')
for src,k,x,y,want in pts:
    if src=='desk' or x==0: continue
    g=eng(x,y)
    if g!=want: print(src,k,x,y,want,g, 'libm:',adjust(math.atan(xdiv(y,x)),x,y))
