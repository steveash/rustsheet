# T-1941 W6: model-free composition detector on P2 (p201..p204). For each column (B=GAMMALN.PRECISE(x), C=LN(x),
# E=GAMMALN.PRECISE(x+1), F=GAMMALN.PRECISE(x-1), G=LN(x-1), H=GAMMA(x)) compute its rounding residual
# (Excel - exact)/ulp and correlate |FFT| of B's residual with the others'. If B is built from another column's kernel
# (even unstored), that kernel's stage lines appear in B's spectrum. Run from the repo root.
import json, sys, numpy as np, mpmath as mp
mp.mp.dps=40
CS='eval/suites/probes-numeric/cases/'
def load(name):
    c=json.load(open(CS+name+'.case.json'))['sheets'][0]['cells']; o=json.load(open(CS+name+'.oracle.json'))['sheets']['Calc']
    R=[];r=1
    while f'A{r}' in c:
        row=[float(c[f'A{r}'])]+[o.get(f'{k}{r}',{}).get('v') for k in 'BCEFGH']; R.append(row); r+=1
    return R
fns={'B':(mp.loggamma,0),'C':(mp.log,0),'E':(mp.loggamma,1),'F':(mp.loggamma,-1),'G':(mp.log,-1),'H':(mp.gamma,0)}
def resid(x,v,f,sh):
    a0=mp.mpf(float(x[0]))+sh; g=[mp.diff(f,a0,k)/mp.factorial(k) for k in range(4)]
    out=[]
    for xi,vi in zip(x,v):
        a=mp.mpf(float(xi))+sh  # exact shifted arg; Excel's own arg is fl(x+-1), close enough for spectra
        d=a-a0; out.append(float((mp.mpf(float(vi))-(g[0]+d*(g[1]+d*(g[2]+d*g[3]))))/mp.mpf(np.spacing(abs(float(vi))))))
    return np.array(out)
def fc(a,c):
    Fa=np.abs(np.fft.rfft(a-a.mean()))[1:]; Fc=np.abs(np.fft.rfft(c-c.mean()))[1:]; return np.corrcoef(Fa,Fc)[0,1]
for nm in sys.argv[1:]:
    R=[r for r in load(nm) if all(isinstance(v,(int,float)) for v in r)]
    A=np.array(R,dtype=float); x=A[:,0]; res={}
    for j,k in enumerate('BCEFGH'):
        f,sh=fns[k]; res[k]=resid(x,A[:,j+1],f,sh)
    print(nm,'n=%d'%len(x),' '.join('B~%s %.3f'%(k,fc(res['B'],res[k])) for k in 'CEFGH'),
          ' | E~C %.3f F~G %.3f'%(fc(res['E'],res['C']),fc(res['F'],res['G'])))
