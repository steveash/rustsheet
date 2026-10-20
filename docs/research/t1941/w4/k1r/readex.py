import k1lib as K, numpy as np, pickle, sys
stages=[(w,int(k)) for w,k in (t.split(':') for t in sys.argv[1].split(','))]   # e.g. D.P:4,N.P:5,N.P:4
pref=sys.argv[2].split(',') if len(sys.argv)>2 else None
sim=K.gsimpy.Sim(K.N0,K.D0); out=[]
for nm,y,g in K.runs(pref):
    base=K.count(sim,y,g)
    rec={'run':nm,'y0':y[0],'n':len(y),'exact':base/len(y)}
    tN=sim.trace('N',y[:1])[0]; tD=sim.trace('D',y[:1])[0]
    for w,k in stages:
        ph,gain,_=K.scan(sim,y,g,w,k)
        tr=tN if w[0]=='N' else tD
        P=tr[k,0]; Sm=tr[k-1,1] if k>0 else np.nan
        rec[(w,k)]=(ph,gain,np.spacing(abs(P)),np.spacing(abs(Sm)) if k>0 else np.nan, Sm, P)
    out.append(rec)
    print('%-6s %.5f %.3f '%(nm,y[0],base/len(y))+' '.join('%s%d %+.3f(%d)'%(w,k+1,rec[(w,k)][0],rec[(w,k)][1]) for w,k in stages),flush=True)
pickle.dump(out,open('readex_%s.pkl'%(sys.argv[3] if len(sys.argv)>3 else 'x'),'wb'))
