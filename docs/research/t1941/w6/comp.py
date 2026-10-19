# T-1941 W6: (1.5,2] composition hypotheses through the (x-2) kernel at x+1, scored by exact rate and by the |FFT|
# correlation between Excel's rounding residual and the simulated one (structure detector, see README).
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import *
mp.mp.dps=40; L2=mp.mpf(np.longdouble(mp.log(2)) if False else 0)
L2=mp.mpf(int(mp.nint(mp.log(2)*2**64)))/2**64   # RN64(ln2): ln2 in [0.5,1) has 64-bit ulp 2^-64
def rn64(v):
    m,e=mp.frexp(v); return mp.ldexp(mp.nint(mp.ldexp(m,64)),e-64)
X,B,C=load(sys.argv[1]); cases=sys.argv[2:]
P,Q=read_k2(); Pg,Qg=read_k2(os.path.join(HERE,'..','w2','grid_a.txt'))
def ld(v): return np.asarray(v,dtype=L)
def horner_ld(c,y):   # y may be longdouble (unstored operand); every op result stored to double
    s=np.full(y.shape,c[0],dtype=np.float64)
    for ci in c[1:]: s=st(L(st(L(s)*y))+L(ci))
    return s
def fc(a,c):
    Fa=np.abs(np.fft.rfft(a-a.mean()))[1:]; Fc=np.abs(np.fft.rfft(c-c.mean()))[1:]; return np.corrcoef(Fa,Fc)[0,1]
for cs in cases:
    s=(C==cs); x=X[s]; b=B[s]; n=len(x)
    x0=mp.mpf(float(x[0])); g=[mp.diff(mp.loggamma,x0,k)/mp.factorial(k) for k in range(4)]
    dx=[mp.mpf(float(xi))-x0 for xi in x]
    ex=np.array([float(g[0]+d*(g[1]+d*(g[2]+d*g[3]))-mp.mpf(float(bi))) for d,bi in zip(dx,b)])
    ub=np.spacing(np.abs(b)); re=-ex/ub
    ln64=np.array([np.longdouble(1)*0+L(float(rn64(L2*mp.log(mp.mpf(float(xi)),2)))) for xi in x],dtype=L)
    # exact 64-bit value: float() would round; rebuild longdouble from mpf via two doubles
    ln64=np.array([L(float(v))+L(float(v-mp.mpf(float(v)))) for v in (rn64(L2*mp.log(mp.mpf(float(xi)),2)) for xi in x)],dtype=L)
    ln53=st(ln64)
    out=[]
    for pn,(PP,QQ) in (('efC',(P,Q)),('grid_a',(Pg,Qg))):
        for yn in ('y53','y64'):
            y=ld(st(L(x)+L(1.0))) if yn=='y53' else L(x)+L(1.0)
            Sv=st(L(horner_ld(PP,y))/L(horner_ld(QQ,y)))
            for mn in ('m53','mex'):
                m=st(y-L(2.0)) if mn=='m53' else y-L(2.0)
                p64=L(m)*L(Sv); p64=ld(p64)   # longdouble product is RN64
                for en in ('E53','E64'):
                    E=ld(st(p64)) if en=='E53' else p64
                    for lnn,LN in (('L53',ld(ln53)),('L64',ln64)):
                        sim=st(E-LN)
                        rs=((sim-b)-ex)/ub
                        out.append((fc(re,rs),(sim==b).mean(),'%s %s %s %s %s'%(pn,yn,mn,en,lnn)))
    out.sort(reverse=True)
    print(cs,'x0=%.5f'%float(x0))
    for f,e,nm in out[:6]: print('   fft %.3f ex %.3f  %s'%(f,e,nm))
