import k1lib as K, numpy as np, pickle, sys
run=sys.argv[1]; fits=[pickle.load(open('lf_%s_%s.pkl'%(run,s),'rb')) for s in sys.argv[2].split(',')]
fits=[f for f in fits if f['score']/f['n']>0.99]
print(run,'good fits:',len(fits),[round(f['score']/f['n'],4) for f in fits])
y0=np.array([fits[0]['y0']])
for ch in ('N','D'):
    tr=[K.gsimpy.Sim(np.array(f['N']),np.array(f['D'])).trace(ch,y0)[0] for f in fits]
    line=[]
    for k in range(7):
        P=[t[k,0] for t in tr]; Sv=[t[k,1] for t in tr]
        line.append('P%d %s S%d %s'%(k+1,'=' if len(set(P))==1 else 'x',k+1,'=' if len(set(Sv))==1 else 'x'))
    print(' ',ch,' '.join(line))
