import k1lib as K, numpy as np, pickle, sys, math
from fractions import Fraction as Fr
runs=sys.argv[1].split(','); out={}
for run in runs:
    fits=[pickle.load(open(f,'rb')) for f in __import__('glob').glob('lf_%s_*.pkl'%run)]
    fits=[f for f in fits if f['score']/f['n']>0.99]
    if len(fits)<2: print(run,'skip (<2 good fits)'); continue
    y0=np.array([fits[0]['y0']])
    for ch in ('N','D'):
        tr=[K.gsimpy.Sim(np.array(f['N']),np.array(f['D'])).trace(ch,y0)[0] for f in fits]
        for k in range(1,7):
            if len(set(t[k,1] for t in tr))==1 and len(set(t[k,0] for t in tr))==1 and len(set(t[k-1,1] for t in tr))==1:
                s=Fr(tr[0][k,1]); p=Fr(tr[0][k,0]); u=Fr(np.spacing(abs(tr[0][k,1])))
                ci=7-(k+1); lo,hi=s-p-u/2,s-p+u/2
                key=(ch,ci); L,H=out.get(key,(None,None))
                out[key]=(lo if L is None else max(L,lo), hi if H is None else min(H,hi))
                print('%s %s%d: [%s, %s]  (width %.3g ulps of c)'%(run,ch,ci,float(lo).hex(),float(hi).hex(),float((hi-lo)/Fr(math.ulp(float(s-p))))))
print('--- intersection, and AN2 offset')
for (ch,ci),(L,H) in sorted(out.items()):
    a=Fr((K.N0 if ch=='N' else K.D0)[ci]); u=Fr(math.ulp(float(a)))
    st='EMPTY' if L>H else 'ok'
    print('%s%d: %s  [%+.2f, %+.2f] ulps from AN2   width %.2f ulps'%(ch,ci,st,float((L-a)/u),float((H-a)/u),float((H-L)/u)))
